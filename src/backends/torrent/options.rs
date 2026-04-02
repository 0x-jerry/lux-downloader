use super::BackendError;
use crate::models::TaskSpec;
use librqbit::AddTorrentOptions;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Default, Deserialize)]
pub(super) struct TorrentProtocolOptions {
    pub(super) disable_trackers: Option<bool>,
    pub(super) trackers: Option<Vec<String>>,
    pub(super) only_files: Option<Vec<usize>>,
    pub(super) only_files_regex: Option<String>,
}

pub(super) fn parse_protocol_options(spec: &TaskSpec) -> Result<TorrentProtocolOptions, BackendError> {
    let Some(protocol_options) = spec.protocol_options.as_ref() else {
        return Ok(TorrentProtocolOptions::default());
    };

    serde_json::from_value(protocol_options.clone()).map_err(|err| {
        BackendError::Unsupported(format!("invalid torrent protocol_options: {err}"))
    })
}

pub(super) fn create_add_torrent_options(
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
