use super::{BackendContext, BackendError, BackendEvent, TransferBackend};
use crate::models::{SourceKind, TaskSpec};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct TorrentBackend;

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
        _spec: TaskSpec,
        _context: BackendContext,
        _cancel: CancellationToken,
        _events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError> {
        Err(BackendError::Unsupported(
            "torrent backend is not implemented yet".to_string(),
        ))
    }
}
