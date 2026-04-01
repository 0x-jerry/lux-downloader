use super::{BackendContext, BackendError, BackendEvent, TransferBackend};
use crate::models::{SourceKind, TaskSpec};
use async_trait::async_trait;
use reqwest::Url;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct SftpBackend;

#[async_trait]
impl TransferBackend for SftpBackend {
    fn name(&self) -> &'static str {
        "sftp"
    }

    fn can_handle(&self, spec: &TaskSpec) -> bool {
        if !matches!(spec.source.kind, SourceKind::Url) {
            return false;
        }

        Url::parse(&spec.source.value)
            .ok()
            .map(|url| matches!(url.scheme(), "sftp"))
            .unwrap_or(false)
    }

    async fn run(
        &self,
        _spec: TaskSpec,
        _context: BackendContext,
        _cancel: CancellationToken,
        _events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError> {
        Err(BackendError::Unsupported(
            "sftp backend is not implemented yet".to_string(),
        ))
    }
}
