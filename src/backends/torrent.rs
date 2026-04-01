use super::{
    BackendContext, BackendError, BackendEvent, TransferBackend, build_client,
    resolve_destination_path,
};
use crate::models::{SourceKind, TaskSpec};
use async_trait::async_trait;
use librqbit::{
    AddTorrent, AddTorrentOptions, AddTorrentResponse, Session, SessionOptions,
    SessionPersistenceConfig,
};
use reqwest::Url;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const PROGRESS_TICK: Duration = Duration::from_secs(1);
const BRIEF_SEEDING_CAP_SECS: u64 = 300;

pub struct TorrentBackend;

#[derive(Debug, Default, Deserialize)]
struct TorrentProtocolOptions {
    disable_dht: Option<bool>,
    disable_trackers: Option<bool>,
    trackers: Option<Vec<String>>,
    only_files: Option<Vec<usize>>,
    only_files_regex: Option<String>,
}

#[async_trait]
impl TransferBackend for TorrentBackend {
    fn name(&self) -> &'static str {
        "torrent"
    }

    fn can_handle(&self, spec: &TaskSpec) -> bool {
        matches!(spec.source.kind, SourceKind::Magnet | SourceKind::Torrent)
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

        let destination_root = resolve_destination_path(&context.download_dir, &spec.destination_path)?;
        tokio::fs::create_dir_all(&destination_root).await?;

        let protocol_options = parse_protocol_options(&spec)?;
        let session = create_session(&context, &destination_root, &protocol_options).await?;
        let add_torrent = create_add_torrent(&spec).await?;
        let add_options = create_add_torrent_options(&spec, &protocol_options, &destination_root);

        let response = session
            .add_torrent(add_torrent, Some(add_options))
            .await
            .map_err(map_torrent_error)?;

        let handle = match response {
            AddTorrentResponse::AlreadyManaged(_, handle) => handle,
            AddTorrentResponse::Added(_, handle) => handle,
            AddTorrentResponse::ListOnly(_) => {
                return Err(BackendError::Torrent(
                    "torrent backend received list-only response unexpectedly".to_string(),
                ));
            }
        };

        let initial_stats = handle.stats();
        let initial_total = (initial_stats.total_bytes > 0).then_some(initial_stats.total_bytes);
        let _ = events.send(BackendEvent::Started {
            total_bytes: initial_total,
        });

        let effective_ratio_limit = effective_ratio_limit(&spec, &context);
        let seeding_secs = effective_brief_seeding_secs(&spec, &context);
        let mut seeding_since: Option<Instant> = None;

        loop {
            if cancel.is_cancelled() {
                let _ = session.pause(&handle).await;
                return Err(BackendError::Cancelled);
            }

            let stats = handle.stats();
            let live = stats.live.as_ref();
            let download_rate_bps = live
                .map(|value| mibps_to_bps(value.download_speed.mbps))
                .unwrap_or(0);
            let upload_rate_bps = live
                .map(|value| mibps_to_bps(value.upload_speed.mbps))
                .unwrap_or(0);

            let _ = events.send(BackendEvent::Progress {
                downloaded_bytes: stats.progress_bytes,
                total_bytes: (stats.total_bytes > 0).then_some(stats.total_bytes),
                download_rate_bps,
                uploaded_bytes: stats.uploaded_bytes,
                upload_rate_bps,
            });

            if stats.finished {
                if seeding_since.is_none() {
                    seeding_since = Some(Instant::now());
                    let _ = events.send(BackendEvent::SeedingStarted);
                }

                if should_finish_seeding(
                    seeding_since,
                    seeding_secs,
                    effective_ratio_limit,
                    stats.uploaded_bytes,
                    stats.total_bytes,
                ) {
                    let _ = session.pause(&handle).await;
                    let _ = events.send(BackendEvent::Progress {
                        downloaded_bytes: stats.progress_bytes,
                        total_bytes: (stats.total_bytes > 0).then_some(stats.total_bytes),
                        download_rate_bps: 0,
                        uploaded_bytes: stats.uploaded_bytes,
                        upload_rate_bps: 0,
                    });
                    let _ = events.send(BackendEvent::Completed);
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
}

async fn create_session(
    context: &BackendContext,
    destination_root: &Path,
    protocol_options: &TorrentProtocolOptions,
) -> Result<std::sync::Arc<Session>, BackendError> {
    let persistence_dir = Path::new(&context.session_dir).join("torrent");
    tokio::fs::create_dir_all(&persistence_dir).await?;

    let session_options = SessionOptions {
        disable_dht: protocol_options.disable_dht.unwrap_or(false),
        fastresume: true,
        persistence: Some(SessionPersistenceConfig::Json {
            folder: Some(persistence_dir),
        }),
        ..Default::default()
    };

    Session::new_with_opts(destination_root.to_path_buf(), session_options)
        .await
        .map_err(map_torrent_error)
}

async fn create_add_torrent(spec: &TaskSpec) -> Result<AddTorrent<'static>, BackendError> {
    match spec.source.kind {
        SourceKind::Magnet => Ok(AddTorrent::from_url(spec.source.value.clone())),
        SourceKind::Torrent => {
            if let Ok(url) = Url::parse(&spec.source.value) {
                if matches!(url.scheme(), "http" | "https") {
                    let client = build_client(spec)?;
                    let bytes = client
                        .get(url)
                        .send()
                        .await?
                        .error_for_status()?
                        .bytes()
                        .await?;
                    return Ok(AddTorrent::from_bytes(bytes));
                }

                return Err(BackendError::Unsupported(format!(
                    "unsupported torrent source url scheme: {}",
                    url.scheme()
                )));
            }

            let bytes = tokio::fs::read(&spec.source.value).await?;
            Ok(AddTorrent::from_bytes(bytes))
        }
        _ => Err(BackendError::Unsupported(format!(
            "unsupported source kind for torrent backend: {:?}",
            spec.source.kind
        ))),
    }
}

fn create_add_torrent_options(
    spec: &TaskSpec,
    protocol_options: &TorrentProtocolOptions,
    destination_root: &Path,
) -> AddTorrentOptions {
    AddTorrentOptions {
        overwrite: spec.overwrite_existing,
        output_folder: Some(destination_root.to_string_lossy().to_string()),
        disable_trackers: protocol_options.disable_trackers.unwrap_or(false),
        trackers: protocol_options.trackers.clone(),
        only_files: protocol_options.only_files.clone(),
        only_files_regex: protocol_options.only_files_regex.clone(),
        ..Default::default()
    }
}

fn parse_protocol_options(spec: &TaskSpec) -> Result<TorrentProtocolOptions, BackendError> {
    let Some(protocol_options) = spec.protocol_options.as_ref() else {
        return Ok(TorrentProtocolOptions::default());
    };

    serde_json::from_value(protocol_options.clone()).map_err(|err| {
        BackendError::Unsupported(format!("invalid torrent protocol_options: {err}"))
    })
}

fn effective_ratio_limit(spec: &TaskSpec, context: &BackendContext) -> Option<f32> {
    spec.settings
        .as_ref()
        .and_then(|settings| settings.seeding_ratio_limit)
        .or(Some(context.default_seeding_ratio_limit))
        .filter(|limit| *limit > 0.0)
}

fn effective_brief_seeding_secs(spec: &TaskSpec, context: &BackendContext) -> u64 {
    spec.settings
        .as_ref()
        .and_then(|settings| settings.seeding_time_limit_secs)
        .unwrap_or(context.default_seeding_time_limit_secs)
        .min(BRIEF_SEEDING_CAP_SECS)
}

fn should_finish_seeding(
    seeding_since: Option<Instant>,
    seeding_secs: u64,
    ratio_limit: Option<f32>,
    uploaded_bytes: u64,
    total_bytes: u64,
) -> bool {
    let elapsed_done = seeding_since
        .map(|since| since.elapsed() >= Duration::from_secs(seeding_secs))
        .unwrap_or(false);

    if elapsed_done {
        return true;
    }

    let Some(limit) = ratio_limit else {
        return false;
    };

    if total_bytes == 0 {
        return false;
    }

    (uploaded_bytes as f64 / total_bytes as f64) >= limit as f64
}

fn mibps_to_bps(mibps: f64) -> u64 {
    if !mibps.is_finite() || mibps <= 0.0 {
        return 0;
    }

    (mibps * 1024.0 * 1024.0) as u64
}

fn map_torrent_error(err: anyhow::Error) -> BackendError {
    BackendError::Torrent(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{BRIEF_SEEDING_CAP_SECS, should_finish_seeding};
    use std::time::Duration;
    use tokio::time::Instant;

    #[test]
    fn completes_when_ratio_reached() {
        let start = Instant::now();
        let finished = should_finish_seeding(Some(start), BRIEF_SEEDING_CAP_SECS, Some(1.0), 100, 100);
        assert!(finished);
    }

    #[test]
    fn completes_when_time_reached() {
        let start = Instant::now() - Duration::from_secs(10);
        let finished = should_finish_seeding(Some(start), 5, None, 0, 100);
        assert!(finished);
    }

    #[test]
    fn keeps_seeding_when_no_limit_reached() {
        let start = Instant::now();
        let finished = should_finish_seeding(Some(start), 300, Some(2.0), 100, 100);
        assert!(!finished);
    }
}
