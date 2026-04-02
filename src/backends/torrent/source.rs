use super::BackendError;
use crate::backends::build_client;
use crate::models::{SourceKind, TaskSpec};
use librqbit::AddTorrent;
use reqwest::Url;

pub(super) async fn create_add_torrent(spec: &TaskSpec) -> Result<AddTorrent<'static>, BackendError> {
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
