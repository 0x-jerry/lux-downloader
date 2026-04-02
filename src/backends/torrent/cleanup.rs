use super::BackendError;
use crate::backends::resolve_destination_path;
use crate::models::TaskSpec;
use std::io::ErrorKind;

pub(super) async fn cleanup_artifacts_folder(
    spec: &TaskSpec,
    download_dir: &str,
) -> Result<(), BackendError> {
    let destination_root = resolve_destination_path(download_dir, &spec.destination_path)?;
    match tokio::fs::remove_dir_all(&destination_root).await {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(BackendError::Io(err)),
    }

    Ok(())
}
