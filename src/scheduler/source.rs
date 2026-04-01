use super::SchedulerError;
use crate::models::{SourceKind, TaskSpec};
use reqwest::Url;

pub(super) fn resolve_source_kind(spec: &mut TaskSpec) -> Result<(), SchedulerError> {
    if spec.source.kind != SourceKind::Auto {
        return Ok(());
    }

    let detected =
        detect_source_kind(&spec.source.value).ok_or(SchedulerError::UnsupportedSource)?;
    spec.source.kind = detected;
    Ok(())
}

fn detect_source_kind(value: &str) -> Option<SourceKind> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("magnet:?") {
        return Some(SourceKind::Magnet);
    }
    if lower.starts_with('<') && lower.contains("<metalink") {
        return Some(SourceKind::Metalink);
    }

    if let Ok(url) = Url::parse(trimmed) {
        let path = url.path().to_ascii_lowercase();
        if path.ends_with(".torrent") {
            return Some(SourceKind::Torrent);
        }
        if path.ends_with(".metalink") || path.ends_with(".meta4") {
            return Some(SourceKind::Metalink);
        }

        return match url.scheme() {
            "http" | "https" | "ftp" | "ftps" | "sftp" => Some(SourceKind::Url),
            _ => None,
        };
    }

    if lower.ends_with(".torrent") {
        return Some(SourceKind::Torrent);
    }
    if lower.ends_with(".metalink") || lower.ends_with(".meta4") {
        return Some(SourceKind::Metalink);
    }

    None
}
