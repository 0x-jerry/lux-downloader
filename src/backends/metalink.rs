use super::{
    BackendContext, BackendError, BackendEvent, HttpFamilyBackend, TransferBackend, build_client,
};
use crate::models::{SourceKind, TaskSpec};
use async_trait::async_trait;
use quick_xml::de::from_str;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct MetalinkBackend;

#[async_trait]
impl TransferBackend for MetalinkBackend {
    fn name(&self) -> &'static str {
        "metalink"
    }

    fn can_handle(&self, spec: &TaskSpec) -> bool {
        matches!(spec.source.kind, SourceKind::Metalink)
    }

    async fn run(
        &self,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
        events: mpsc::UnboundedSender<BackendEvent>,
    ) -> Result<(), BackendError> {
        let client = build_client(&spec)?;

        let xml = if spec.source.value.trim_start().starts_with('<') {
            spec.source.value.clone()
        } else {
            client
                .get(&spec.source.value)
                .send()
                .await?
                .error_for_status()?
                .text()
                .await?
        };

        let mirror = first_metalink_mirror(&xml)?;
        let mut delegated = spec.clone();
        delegated.source.kind = SourceKind::Url;
        delegated.source.value = mirror;

        HttpFamilyBackend
            .run(delegated, context, cancel, events)
            .await
    }
}

#[derive(Debug, Deserialize)]
struct Metalink {
    #[serde(rename = "file")]
    files: Vec<MetalinkFile>,
}

#[derive(Debug, Deserialize)]
struct MetalinkFile {
    #[serde(rename = "url", default)]
    urls: Vec<String>,
}

fn first_metalink_mirror(xml: &str) -> Result<String, BackendError> {
    let parsed: Metalink = from_str(xml)?;
    parsed
        .files
        .into_iter()
        .flat_map(|file| file.urls)
        .map(|url| url.trim().to_string())
        .find(|url| !url.is_empty())
        .ok_or(BackendError::NoMetalinkMirror)
}
