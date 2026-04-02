use super::{BackendContext, BackendError, map_torrent_error};
use librqbit::{Session, SessionOptions, SessionPersistenceConfig};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub(super) async fn create_session(
    session: &OnceCell<Arc<Session>>,
    context: &BackendContext,
) -> Result<Arc<Session>, BackendError> {
    session
        .get_or_try_init(|| async {
            let persistence_dir = Path::new(&context.session_dir).join("torrent");
            tokio::fs::create_dir_all(&persistence_dir).await?;

            let session_options = SessionOptions {
                fastresume: true,
                persistence: Some(SessionPersistenceConfig::Json {
                    folder: Some(persistence_dir),
                }),
                ..Default::default()
            };

            Session::new_with_opts(Path::new(&context.download_dir).to_path_buf(), session_options)
                .await
                .map_err(map_torrent_error)
        })
        .await
        .cloned()
}
