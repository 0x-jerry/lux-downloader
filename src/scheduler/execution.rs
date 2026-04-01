use super::{Scheduler, SchedulerError};
use crate::backends::{BackendContext, BackendError, BackendEvent, TransferBackend};
use crate::models::{TaskSpec, TaskState};
use crate::persistence::to_json_value;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

impl Scheduler {
    pub(super) async fn start_task_execution(&self, id: Uuid) -> Result<(), SchedulerError> {
        {
            let guard = self.active.read().await;
            if guard.contains_key(&id) {
                return Err(SchedulerError::AlreadyRunning);
            }
        }

        let (backend, spec, context) = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(&id).ok_or(SchedulerError::NotFound)?;
            let backend = self
                .find_backend(&task.spec)
                .ok_or(SchedulerError::UnsupportedSource)?;
            let settings = self.settings.read().await;
            let context = BackendContext {
                download_dir: settings.download_dir.clone(),
                http_chunk_size_bytes: settings.http_chunk_size_bytes,
            };
            (backend, task.spec.clone(), context)
        };

        let cancel = CancellationToken::new();
        {
            let mut guard = self.active.write().await;
            guard.insert(id, cancel.clone());
        }

        let scheduler = self.clone();
        tokio::spawn(async move {
            scheduler
                .run_backend_task(id, backend, spec, context, cancel)
                .await;
        });

        Ok(())
    }

    async fn run_backend_task(
        &self,
        id: Uuid,
        backend: Arc<dyn TransferBackend>,
        spec: TaskSpec,
        context: BackendContext,
        cancel: CancellationToken,
    ) {
        let (tx, mut rx) = mpsc::unbounded_channel::<BackendEvent>();
        let join = tokio::spawn(async move { backend.run(spec, context, cancel, tx).await });

        while let Some(event) = rx.recv().await {
            match event {
                BackendEvent::Started { total_bytes } => {
                    let _ = self
                        .set_download_started(id, total_bytes)
                        .await
                        .map_err(|err| self.emit_error(id, &err.to_string()));
                }
                BackendEvent::Progress {
                    downloaded_bytes,
                    total_bytes,
                    download_rate_bps,
                } => {
                    let _ = self
                        .set_progress(id, downloaded_bytes, total_bytes, download_rate_bps)
                        .await
                        .map_err(|err| self.emit_error(id, &err.to_string()));
                }
                BackendEvent::Completed => {
                    let _ = self
                        .set_completed(id)
                        .await
                        .map_err(|err| self.emit_error(id, &err.to_string()));
                }
            }
        }

        let result = match join.await {
            Ok(result) => result,
            Err(err) => Err(BackendError::Unsupported(format!(
                "backend task panicked: {err}"
            ))),
        };

        if let Err(err) = result {
            match err {
                BackendError::Cancelled => {}
                other => {
                    let _ = self
                        .set_failed(id, other.to_string())
                        .await
                        .map_err(|persist_err| self.emit_error(id, &persist_err.to_string()));
                }
            }
        }

        let mut active = self.active.write().await;
        active.remove(&id);
    }

    async fn set_download_started(
        &self,
        id: Uuid,
        total_bytes: Option<u64>,
    ) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        let Some(task) = tasks.get_mut(&id) else {
            return Ok(());
        };

        if matches!(task.state, TaskState::Paused | TaskState::Removed) {
            return Ok(());
        }

        task.state = TaskState::Downloading;
        task.progress.total_bytes = total_bytes;
        task.updated_at = Utc::now();
        let cloned = task.clone();
        drop(tasks);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_started", to_json_value(&cloned));
        Ok(())
    }

    async fn set_progress(
        &self,
        id: Uuid,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        download_rate_bps: u64,
    ) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        let Some(task) = tasks.get_mut(&id) else {
            return Ok(());
        };

        if task.state != TaskState::Downloading {
            return Ok(());
        }

        task.progress.downloaded_bytes = downloaded_bytes;
        if total_bytes.is_some() {
            task.progress.total_bytes = total_bytes;
        }
        task.progress.download_rate_bps = download_rate_bps;
        task.updated_at = Utc::now();
        let cloned = task.clone();
        drop(tasks);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_progress", to_json_value(&cloned));
        Ok(())
    }

    async fn set_completed(&self, id: Uuid) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        let Some(task) = tasks.get_mut(&id) else {
            return Ok(());
        };

        if matches!(task.state, TaskState::Paused | TaskState::Removed) {
            return Ok(());
        }

        task.state = TaskState::Completed;
        task.progress.download_rate_bps = 0;
        task.progress.upload_rate_bps = 0;
        task.progress.verified = true;
        if let Some(total) = task.progress.total_bytes {
            task.progress.downloaded_bytes = total;
        }
        task.updated_at = Utc::now();
        task.error = None;
        let cloned = task.clone();
        drop(tasks);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_completed", to_json_value(&cloned));
        Ok(())
    }

    async fn set_failed(&self, id: Uuid, message: String) -> Result<(), SchedulerError> {
        let mut tasks = self.tasks.write().await;
        let Some(task) = tasks.get_mut(&id) else {
            return Ok(());
        };

        if matches!(task.state, TaskState::Paused | TaskState::Removed) {
            return Ok(());
        }

        task.state = TaskState::Failed;
        task.progress.download_rate_bps = 0;
        task.progress.upload_rate_bps = 0;
        task.updated_at = Utc::now();
        task.error = Some(message);
        let cloned = task.clone();
        drop(tasks);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_failed", to_json_value(&cloned));
        Ok(())
    }

    pub(super) async fn cancel_task(&self, id: Uuid) {
        let mut guard = self.active.write().await;
        if let Some(token) = guard.remove(&id) {
            token.cancel();
        }
    }

    fn emit_error(&self, id: Uuid, message: &str) {
        self.emit(
            Some(id),
            "task_warning",
            json!({
                "task_id": id,
                "message": message,
            }),
        );
    }
}
