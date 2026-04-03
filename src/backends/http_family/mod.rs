use super::{BackendContext, BackendError, BackendEvent, TransferBackend, build_client};
use crate::{
    backends::resolve_destination_path,
    models::{SourceKind, TaskSpec},
};
use async_trait::async_trait;
use reqwest::Url;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

mod io;
mod multi;
mod probe;
mod single;

const MAX_THREADS: u32 = 32;
const MIN_HTTP_CHUNK_SIZE_BYTES: u64 = 64 * 1024;

pub struct HttpFamilyBackend;

#[derive(Debug)]
enum DownloadPlan {
    Single,
    Multi { total_bytes: u64, thread_count: u32 },
}

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
        let plan =
            plan_download_mode(&client, &url, desired_threads, existing_bytes, chunk_size).await?;

        let result = match plan {
            DownloadPlan::Multi {
                total_bytes,
                thread_count,
            } => multi::download_multi_chunked(
                client,
                url.clone(),
                destination.clone(),
                total_bytes,
                thread_count,
                chunk_size,
                cancel,
                events,
            )
            .await,
            DownloadPlan::Single => {
                single::download_single_stream(
                    client,
                    url.clone(),
                    destination.clone(),
                    existing_bytes,
                    chunk_size,
                    cancel,
                    events,
                )
                .await
            }
        };

        if let Err(err) = &result {
            if !matches!(err, BackendError::Cancelled) {
                error!(
                    %url,
                    destination = %destination.display(),
                    error = %err,
                    "http download failed"
                );
            }
        }

        result
    }
}

async fn plan_download_mode(
    client: &reqwest::Client,
    url: &Url,
    desired_threads: u32,
    existing_bytes: u64,
    chunk_size: u64,
) -> Result<DownloadPlan, BackendError> {
    if desired_threads <= 1 || existing_bytes > 0 {
        return Ok(DownloadPlan::Single);
    }

    let total_bytes = match probe::probe_range_support(client, url).await? {
        Some(bytes) => bytes,
        None => return Ok(DownloadPlan::Single),
    };

    let thread_count = compute_thread_count(desired_threads, total_bytes, chunk_size);
    if thread_count > 1 {
        debug!(%url, total_bytes, thread_count, "http download will use multi-thread mode");
        Ok(DownloadPlan::Multi {
            total_bytes,
            thread_count,
        })
    } else {
        Ok(DownloadPlan::Single)
    }
}

fn compute_thread_count(desired: u32, total_bytes: u64, chunk_size: u64) -> u32 {
    if total_bytes <= chunk_size {
        return 1;
    }

    let chunk_count = total_bytes.div_ceil(chunk_size) as u32;
    desired.min(chunk_count).max(1)
}
