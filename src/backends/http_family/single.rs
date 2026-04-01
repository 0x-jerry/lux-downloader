use super::io;
use crate::backends::{BackendError, BackendEvent};
use reqwest::Url;
use reqwest::header::RANGE;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub async fn download_single_stream(
    client: reqwest::Client,
    url: Url,
    destination: PathBuf,
    existing_bytes: u64,
    chunk_size: u64,
    cancel: CancellationToken,
    events: mpsc::UnboundedSender<BackendEvent>,
) -> Result<(), BackendError> {
    let mut request = client.get(url);
    if existing_bytes > 0 {
        request = request.header(RANGE, format!("bytes={existing_bytes}-"));
    }

    let response = request.send().await?.error_for_status()?;
    let resumed = existing_bytes > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;

    let total_bytes = response
        .content_length()
        .map(|len| if resumed { len + existing_bytes } else { len });
    let _ = events.send(BackendEvent::Started { total_bytes });

    let writer_capacity = chunk_size.clamp(64 * 1024, 2 * 1024 * 1024) as usize;
    let mut file = BufWriter::with_capacity(
        writer_capacity,
        io::open_destination(&destination, resumed).await?,
    );

    let mut downloaded = if resumed { existing_bytes } else { 0 };
    let mut checkpoint_bytes = downloaded;
    let mut checkpoint_time = Instant::now();
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        if cancel.is_cancelled() {
            return Err(BackendError::Cancelled);
        }

        let chunk = chunk?;
        if chunk_size >= chunk.len() as u64 {
            file.write_all(&chunk).await?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            emit_progress(
                &events,
                downloaded,
                total_bytes,
                &mut checkpoint_bytes,
                &mut checkpoint_time,
            );
            continue;
        }

        for part in chunk.chunks(chunk_size as usize) {
            file.write_all(part).await?;
            downloaded = downloaded.saturating_add(part.len() as u64);
            emit_progress(
                &events,
                downloaded,
                total_bytes,
                &mut checkpoint_bytes,
                &mut checkpoint_time,
            );
        }
    }

    file.flush().await?;
    let _ = events.send(BackendEvent::Completed);
    Ok(())
}

fn emit_progress(
    events: &mpsc::UnboundedSender<BackendEvent>,
    downloaded: u64,
    total_bytes: Option<u64>,
    checkpoint_bytes: &mut u64,
    checkpoint_time: &mut Instant,
) {
    let now = Instant::now();
    let elapsed = now.duration_since(*checkpoint_time);
    let rate = if elapsed.as_secs_f64() > 0.0 {
        ((downloaded.saturating_sub(*checkpoint_bytes)) as f64 / elapsed.as_secs_f64()) as u64
    } else {
        0
    };

    if elapsed.as_millis() >= 400 {
        *checkpoint_time = now;
        *checkpoint_bytes = downloaded;
    }

    let _ = events.send(BackendEvent::Progress {
        downloaded_bytes: downloaded,
        total_bytes,
        download_rate_bps: rate,
    });
}
