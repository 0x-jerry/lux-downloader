use super::{BackendContext, BackendError, BackendEvent, TransferBackend};
use crate::backends::resolve_destination_path;
use crate::models::{SourceKind, TaskSpec};
use async_trait::async_trait;
use librqbit::{ManagedTorrent, Session, TorrentStats, TorrentStatsState};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OnceCell, mpsc};
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

mod cleanup;
mod options;
mod seeding;
mod session;
mod source;

const PROGRESS_TICK: Duration = Duration::from_secs(1);

pub struct TorrentBackend {
    session: OnceCell<Arc<Session>>,
}

impl Default for TorrentBackend {
    fn default() -> Self {
        Self {
            session: OnceCell::const_new(),
        }
    }
}

#[async_trait]
impl TransferBackend for TorrentBackend {
    fn name(&self) -> &'static str {
        "torrent"
    }

    fn can_handle(&self, spec: &TaskSpec) -> bool {
        matches!(spec.source.kind, SourceKind::Magnet | SourceKind::Torrent)
    }

    async fn init(&self, context: &BackendContext) -> Result<(), BackendError> {
        let _ = session::create_session(&self.session, context).await?;
        Ok(())
    }

    async fn run(
        &self,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
        events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError> {
        if cancel.is_cancelled() {
            return Err(BackendError::Cancelled);
        }

        let destination_root =
            resolve_destination_path(&context.download_dir, &spec.destination_path)?;
        tokio::fs::create_dir_all(&destination_root).await?;

        let protocol_options = options::parse_protocol_options(&spec)?;
        let session =
            self.session.get().cloned().ok_or_else(|| {
                BackendError::Torrent("torrent backend not initialized".to_string())
            })?;
        let handle =
            resolve_or_add_handle(&spec, &protocol_options, &destination_root, &session).await?;
        let _ = events.send(BackendEvent::TorrentIdAssigned {
            torrent_id: handle.id().into(),
        });

        let initial_stats = handle.stats();
        let initial_total = total_bytes_option(initial_stats.total_bytes);
        let _ = events.send(BackendEvent::Started {
            total_bytes: initial_total,
        });

        let effective_ratio_limit = seeding::effective_ratio_limit(&spec, &context);
        let seeding_secs = seeding::effective_brief_seeding_secs(&spec, &context);
        let mut seeding_since: Option<Instant> = None;

        if matches!(initial_stats.state, TorrentStatsState::Paused) {
            let _ = session.unpause(&handle).await;
        };

        if initial_stats.finished {
            complete_and_delete(&session, &handle, &events, None).await;
            return Ok(());
        }

        loop {
            if cancel.is_cancelled() {
                let _ = session.pause(&handle).await;
                return Err(BackendError::Cancelled);
            }

            let stats = handle.stats();
            emit_progress(&events, &stats);

            if stats.finished {
                if seeding_since.is_none() {
                    seeding_since = Some(Instant::now());
                    let _ = events.send(BackendEvent::SeedingStarted);
                }

                if seeding::should_finish_seeding(
                    seeding_since,
                    seeding_secs,
                    effective_ratio_limit,
                    stats.uploaded_bytes,
                    stats.total_bytes,
                ) {
                    let _ = session.pause(&handle).await;
                    complete_and_delete(&session, &handle, &events, Some(&stats)).await;
                    return Ok(());
                }
            }

            tokio::select! {
                _ = cancel.cancelled() => {
                    let _ = session.pause(&handle).await;
                    return Err(BackendError::Cancelled);
                }
                _ = tokio::time::sleep(PROGRESS_TICK) => {}
            }
        }
    }

    async fn cleanup(&self, spec: &TaskSpec, context: &BackendContext) -> Result<(), BackendError> {
        let session =
            self.session.get().cloned().ok_or_else(|| {
                BackendError::Torrent("torrent backend not initialized".to_string())
            })?;
        if let Some(torrent_id) = torrent_id_from_spec(spec) {
            let _ = session.delete(torrent_id.into(), true).await;
        }

        let _ = cleanup::cleanup_artifacts_folder(spec, &context.download_dir).await;

        Ok(())
    }
}

pub(super) fn map_torrent_error(err: anyhow::Error) -> BackendError {
    BackendError::Torrent(err.to_string())
}

async fn resolve_or_add_handle(
    spec: &TaskSpec,
    protocol_options: &options::TorrentProtocolOptions,
    destination_root: &Path,
    session: &Arc<Session>,
) -> Result<Arc<ManagedTorrent>, BackendError> {
    if let Some(existing) =
        torrent_id_from_spec(spec).and_then(|torrent_id| session.get(torrent_id.into()))
    {
        return Ok(existing);
    }

    let add_torrent = source::create_add_torrent(spec).await?;
    let add_options = options::create_add_torrent_options(spec, protocol_options, destination_root);
    let response = session
        .add_torrent(add_torrent, Some(add_options))
        .await
        .map_err(map_torrent_error)?;

    response.into_handle().ok_or_else(|| {
        BackendError::Torrent(
            "torrent backend received list-only response unexpectedly".to_string(),
        )
    })
}

fn emit_progress(events: &mpsc::UnboundedSender<BackendEvent>, stats: &TorrentStats) {
    let live = stats.live.as_ref();
    let download_rate_bps = live
        .map(|value| seeding::mibps_to_bps(value.download_speed.mbps))
        .unwrap_or(0);
    let upload_rate_bps = live
        .map(|value| seeding::mibps_to_bps(value.upload_speed.mbps))
        .unwrap_or(0);
    let _ = events.send(BackendEvent::Progress {
        downloaded_bytes: stats.progress_bytes,
        total_bytes: total_bytes_option(stats.total_bytes),
        download_rate_bps,
        uploaded_bytes: stats.uploaded_bytes,
        upload_rate_bps,
    });
}

async fn complete_and_delete(
    session: &Arc<Session>,
    handle: &Arc<ManagedTorrent>,
    events: &mpsc::UnboundedSender<BackendEvent>,
    final_stats: Option<&TorrentStats>,
) {
    let _ = session.delete(handle.id().into(), false).await;
    if let Some(stats) = final_stats {
        let _ = events.send(BackendEvent::Progress {
            downloaded_bytes: stats.progress_bytes,
            total_bytes: total_bytes_option(stats.total_bytes),
            download_rate_bps: 0,
            uploaded_bytes: stats.uploaded_bytes,
            upload_rate_bps: 0,
        });
    }
    let _ = events.send(BackendEvent::Completed);
}

fn total_bytes_option(total_bytes: u64) -> Option<u64> {
    (total_bytes > 0).then_some(total_bytes)
}

fn torrent_id_from_spec(spec: &TaskSpec) -> Option<usize> {
    let options = spec.protocol_options.as_ref()?;
    let torrent_id = options.get("torrent_id")?;
    let torrent_id = torrent_id.as_u64()?;
    usize::try_from(torrent_id).ok()
}
