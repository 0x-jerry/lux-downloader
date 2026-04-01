mod ftp;
mod http_family;
mod metalink;
mod sftp;
mod torrent;

pub use ftp::FtpBackend;
pub use http_family::HttpFamilyBackend;
pub use metalink::MetalinkBackend;
pub use sftp::SftpBackend;
pub use torrent::TorrentBackend;

use crate::models::TaskSpec;
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client, Proxy};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone)]
pub enum BackendEvent {
    Started {
        total_bytes: Option<u64>,
    },
    Progress {
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        download_rate_bps: u64,
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
}

#[derive(Clone)]
pub struct BackendContext {
    pub download_dir: String,
    pub http_chunk_size_bytes: u64,
}

#[async_trait]
pub trait TransferBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, spec: &TaskSpec) -> bool;
    async fn run(
        &self,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
        events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError>;
}

pub fn default_backends() -> Vec<Arc<dyn TransferBackend>> {
    vec![
        Arc::new(HttpFamilyBackend),
        Arc::new(FtpBackend),
        Arc::new(SftpBackend),
        Arc::new(TorrentBackend),
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

pub(crate) fn resolve_destination_path(
    download_dir: &str,
    destination: &str,
) -> Result<PathBuf, BackendError> {
    if destination.trim().is_empty() {
        return Err(BackendError::Unsupported(
            "destination_path cannot be empty".to_string(),
        ));
    }

    let mut relative = PathBuf::new();
    for component in Path::new(destination).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => relative.push(part),
            Component::ParentDir => {
                return Err(BackendError::Unsupported(
                    "destination_path must not contain '..'".to_string(),
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(BackendError::Unsupported(
                    "destination_path must be relative to download_dir".to_string(),
                ));
            }
        }
    }

    if relative.as_os_str().is_empty() {
        return Err(BackendError::Unsupported(
            "destination_path cannot resolve to current directory".to_string(),
        ));
    }

    Ok(Path::new(download_dir).join(relative))
}
