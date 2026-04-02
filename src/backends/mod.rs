mod ftp;
mod http_family;
mod metalink;
mod sftp;
mod torrent;

pub use ftp::FtpBackend;
pub use http_family::HttpFamilyBackend;
pub use metalink::MetalinkBackend;
use path_clean::PathClean;
pub use sftp::SftpBackend;
pub use torrent::TorrentBackend;

use crate::models::TaskSpec;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Proxy};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum BackendEvent {
    TorrentIdAssigned {
        torrent_id: usize,
    },
    Started {
        total_bytes: Option<u64>,
    },
    SeedingStarted,
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        download_rate_bps: u64,
        uploaded_bytes: u64,
        upload_rate_bps: u64,
    },
    Completed,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("unsupported source for backend: {0}")]
    Unsupported(String),
    #[error("cancelled")]
    Cancelled,
    #[error("invalid source url: {0}")]
    InvalidUrl(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("xml parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("metalink did not contain any usable mirrors")]
    NoMetalinkMirror,
    #[error("torrent backend error: {0}")]
    Torrent(String),
}

#[derive(Clone)]
pub struct BackendContext {
    pub download_dir: String,
    pub session_dir: String,
    pub http_chunk_size_bytes: u64,
    pub default_seeding_ratio_limit: f32,
    pub default_seeding_time_limit_secs: u64,
}

#[async_trait]
pub trait TransferBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, spec: &TaskSpec) -> bool;
    async fn init(&self, _context: &BackendContext) -> Result<(), BackendError> {
        Ok(())
    }
    async fn run(
        &self,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
        events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError>;

    async fn cleanup(
        &self,
        _spec: &TaskSpec,
        _context: &BackendContext,
    ) -> Result<(), BackendError> {
        Ok(())
    }

    async fn torrent_stats(
        &self,
        _spec: &TaskSpec,
        _context: &BackendContext,
    ) -> Result<Option<Value>, BackendError> {
        Ok(None)
    }
}

pub fn default_backends() -> Vec<Arc<dyn TransferBackend>> {
    vec![
        Arc::new(HttpFamilyBackend),
        Arc::new(FtpBackend),
        Arc::new(SftpBackend),
        Arc::new(TorrentBackend::default()),
        Arc::new(MetalinkBackend),
    ]
}

pub(crate) fn build_client(spec: &TaskSpec) -> Result<Client, BackendError> {
    let mut builder = Client::builder()
        .gzip(true)
        .redirect(reqwest::redirect::Policy::limited(10));

    if let Some(settings) = spec.settings.as_ref() {
        if let Some(proxy) = settings.proxy.as_ref() {
            builder = builder.proxy(Proxy::all(proxy).map_err(BackendError::Http)?);
        }

        if let Some(headers) = settings.headers.as_ref() {
            let mut map = HeaderMap::new();
            for pair in headers {
                let name = HeaderName::from_bytes(pair.name.as_bytes()).map_err(|_| {
                    BackendError::Unsupported(format!("invalid header name {}", pair.name))
                })?;
                let value = HeaderValue::from_str(&pair.value).map_err(|_| {
                    BackendError::Unsupported(format!("invalid header value for {}", pair.name))
                })?;
                map.insert(name, value);
            }
            builder = builder.default_headers(map);
        }
    }

    Ok(builder.build()?)
}

pub(crate) fn check_destination_path(destination: &str) -> Result<PathBuf, BackendError> {
    if destination.trim().is_empty() {
        return Err(BackendError::Unsupported(
            "destination_path cannot be empty".to_string(),
        ));
    }

    let destination = Path::new(destination).clean();

    if destination.is_absolute() || destination.starts_with("..") {
        return Err(BackendError::Unsupported(
            "destination_path must stay inside download_dir".to_string(),
        ));
    }

    Ok(destination)
}

pub(crate) fn resolve_destination_path(
    download_dir: &str,
    destination: &str,
) -> Result<PathBuf, BackendError> {
    let dest = Path::new(download_dir).join(destination);

    Ok(dest)
}
