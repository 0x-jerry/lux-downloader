use super::{BackendContext, BackendError, BackendEvent, TransferBackend, build_client};
use crate::{
    backends::resolve_destination_path,
    models::{SourceKind, TaskSpec},
};
use async_trait::async_trait;
use reqwest::Url;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

mod io;
mod multi;
mod probe;
mod single;

const MAX_THREADS: u32 = 32;
const MIN_HTTP_CHUNK_SIZE_BYTES: u64 = 64 * 1024;

pub struct HttpFamilyBackend;

#[async_trait]
impl TransferBackend for HttpFamilyBackend {
    fn name(&self) -> &'static str {
        "http_family"
    }

    fn can_handle(&self, spec: &TaskSpec) -> bool {
        if !matches!(spec.source.kind, SourceKind::Url) {
            return false;
        }

        Url::parse(&spec.source.value)
            .ok()
            .map(|url| matches!(url.scheme(), "http" | "https"))
            .unwrap_or(false)
    }

    async fn run(
        &self,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
        events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError> {
        let client = build_client(&spec)?;
        let url = Url::parse(&spec.source.value)
            .map_err(|_| BackendError::InvalidUrl(spec.source.value.clone()))?;

        let destination = resolve_destination_path(&context.download_dir, &spec.destination_path)?;
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let chunk_size = context.http_chunk_size_bytes.max(MIN_HTTP_CHUNK_SIZE_BYTES);
        let existing_bytes = if spec.overwrite_existing {
            0
        } else {
            io::file_size(&destination).await?
        };
        let desired_threads = spec.concurrency.unwrap_or(1).clamp(1, MAX_THREADS);

        if desired_threads > 1 && existing_bytes == 0 {
            if let Some(total_bytes) = probe::probe_range_support(&client, &url).await? {
                let thread_count = compute_thread_count(desired_threads, total_bytes, chunk_size);
                if thread_count > 1 {
                    return multi::download_multi_chunked(
                        client,
                        url,
                        destination,
                        total_bytes,
                        thread_count,
                        chunk_size,
                        cancel,
                        events,
                    )
                    .await;
                }
            }
        }

        single::download_single_stream(
            client,
            url,
            destination,
            existing_bytes,
            chunk_size,
            cancel,
            events,
        )
        .await
    }
}

fn compute_thread_count(desired: u32, total_bytes: u64, chunk_size: u64) -> u32 {
    if total_bytes <= chunk_size {
        return 1;
    }

    let chunk_count = total_bytes.div_ceil(chunk_size) as u32;
    desired.min(chunk_count).max(1)
}
