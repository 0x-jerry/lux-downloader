use super::io;
use crate::backends::{BackendError, BackendEvent};
use reqwest::Url;
use reqwest::header::RANGE;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter, SeekFrom};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub async fn download_multi_chunked(
    client: reqwest::Client,
    url: Url,
    destination: PathBuf,
    total_bytes: u64,
    thread_count: u32,
    chunk_size: u64,
    cancel: CancellationToken,
    events: mpsc::UnboundedSender<BackendEvent>,
) -> Result<(), BackendError> {
    let _ = events.send(BackendEvent::Started {
        total_bytes: Some(total_bytes),
    });

    let chunks = Arc::new(split_ranges(total_bytes, chunk_size));
    io::create_preallocated(&destination, total_bytes).await?;
    let next_index = Arc::new(AtomicUsize::new(0));

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<u64>();
    let mut joins = JoinSet::new();

    for _ in 0..thread_count {
        let worker_client = client.clone();
        let worker_url = url.clone();
        let worker_destination = destination.clone();
        let worker_chunks = chunks.clone();
        let worker_next_index = next_index.clone();
        let worker_cancel = cancel.clone();
        let worker_progress = progress_tx.clone();

        joins.spawn(async move {
            download_chunks_to_destination(
                worker_client,
                worker_url,
                worker_destination,
                worker_chunks,
                worker_next_index,
                worker_cancel,
                worker_progress,
            )
            .await
        });
    }
    drop(progress_tx);

    let mut downloaded = 0_u64;
    let mut checkpoint_bytes = 0_u64;
    let mut checkpoint_time = Instant::now();
    let mut first_error: Option<BackendError> = None;
    let mut workers_left = joins.len();

    while workers_left > 0 {
        tokio::select! {
            chunk = progress_rx.recv() => {
                if let Some(delta) = chunk {
                    downloaded = downloaded.saturating_add(delta);
                    let now = Instant::now();
                    let elapsed = now.duration_since(checkpoint_time);
                    let rate = if elapsed.as_secs_f64() > 0.0 {
                        ((downloaded.saturating_sub(checkpoint_bytes)) as f64 / elapsed.as_secs_f64()) as u64
                    } else {
                        0
                    };

                    if elapsed.as_millis() >= 400 {
                        checkpoint_time = now;
                        checkpoint_bytes = downloaded;
                    }

                    let _ = events.send(BackendEvent::Progress {
                        downloaded_bytes: downloaded,
                        total_bytes: Some(total_bytes),
                        download_rate_bps: rate,
                        uploaded_bytes: 0,
                        upload_rate_bps: 0,
                    });
                }
            }
            joined = joins.join_next() => {
                if let Some(result) = joined {
                    workers_left -= 1;
                    match result {
                        Ok(Ok(())) => {}
                        Ok(Err(BackendError::Cancelled)) => {}
                        Ok(Err(err)) => {
                            if first_error.is_none() {
                                first_error = Some(err);
                            }
                            cancel.cancel();
                        }
                        Err(err) => {
                            if first_error.is_none() {
                                first_error = Some(BackendError::Unsupported(format!(
                                    "worker join error: {err}"
                                )));
                            }
                            cancel.cancel();
                        }
                    }
                }
            }
        }
    }

    if cancel.is_cancelled() && first_error.is_none() {
        return Err(BackendError::Cancelled);
    }

    if let Some(err) = first_error {
        return Err(err);
    }

    let _ = events.send(BackendEvent::Progress {
        downloaded_bytes: total_bytes,
        total_bytes: Some(total_bytes),
        download_rate_bps: 0,
        uploaded_bytes: 0,
        upload_rate_bps: 0,
    });
    let _ = events.send(BackendEvent::Completed);
    Ok(())
}

async fn download_chunks_to_destination(
    client: reqwest::Client,
    url: Url,
    destination: PathBuf,
    chunks: Arc<Vec<(u64, u64)>>,
    next_index: Arc<AtomicUsize>,
    cancel: CancellationToken,
    progress_tx: mpsc::UnboundedSender<u64>,
) -> Result<(), BackendError> {
    let mut writer =
        BufWriter::with_capacity(256 * 1024, io::open_existing_writable(&destination).await?);

    loop {
        if cancel.is_cancelled() {
            return Err(BackendError::Cancelled);
        }

        let idx = next_index.fetch_add(1, Ordering::Relaxed);
        if idx >= chunks.len() {
            break;
        }

        let (start, end) = chunks[idx];
        let data = download_range(client.clone(), url.clone(), start, end, cancel.clone()).await?;

        writer.seek(SeekFrom::Start(start)).await?;
        writer.write_all(&data).await?;
        writer.flush().await?;
        let _ = progress_tx.send(data.len() as u64);
    }

    writer.flush().await?;
    Ok(())
}

async fn download_range(
    client: reqwest::Client,
    url: Url,
    start: u64,
    end: u64,
    cancel: CancellationToken,
) -> Result<Vec<u8>, BackendError> {
    let response = client
        .get(url)
        .header(RANGE, format!("bytes={start}-{end}"))
        .send()
        .await?
        .error_for_status()?;

    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(BackendError::Unsupported(
            "server did not honor range requests".to_string(),
        ));
    }

    let expected_size = end - start + 1;
    let mut output = Vec::with_capacity(expected_size as usize);
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        if cancel.is_cancelled() {
            return Err(BackendError::Cancelled);
        }
        output.extend_from_slice(&chunk?);
    }

    if output.len() as u64 != expected_size {
        return Err(BackendError::Unsupported(format!(
            "range response length mismatch for {start}-{end}"
        )));
    }

    Ok(output)
}

fn split_ranges(total_bytes: u64, chunk_size: u64) -> Vec<(u64, u64)> {
    if total_bytes == 0 {
        return Vec::new();
    }

    let mut ranges = Vec::with_capacity(total_bytes.div_ceil(chunk_size) as usize);
    let mut start = 0_u64;

    while start < total_bytes {
        let end = (start + chunk_size - 1).min(total_bytes - 1);
        ranges.push((start, end));
        start = end + 1;
    }

    ranges
}
