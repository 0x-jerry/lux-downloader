use crate::backends::BackendError;
use std::path::Path;
use tokio::fs::{self, File, OpenOptions};

pub async fn file_size(path: &Path) -> Result<u64, BackendError> {
    match fs::metadata(path).await {
        Ok(meta) => Ok(meta.len()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(BackendError::Io(err)),
    }
}

pub async fn create_preallocated(path: &Path, bytes: u64) -> Result<(), BackendError> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .await?;
    file.set_len(bytes).await?;
    Ok(())
}

pub async fn open_destination(path: &Path, append: bool) -> Result<File, BackendError> {
    if append {
        Ok(OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?)
    } else {
        Ok(OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .await?)
    }
}

pub async fn open_existing_writable(path: &Path) -> Result<File, BackendError> {
    Ok(OpenOptions::new()
        .create(true)
        .write(true)
        .open(path)
        .await?)
}
