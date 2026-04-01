use super::source::resolve_source_kind;
use super::transitions::validate_transition;
use super::{Scheduler, SchedulerError};
use crate::backends::resolve_destination_path;
use crate::models::{
    GlobalSettings, SourceKind, TaskListQuery, TaskPatch, TaskProgress, TaskRuntimeSettings,
    TaskSpec, TaskState, TaskView,
};
use crate::persistence::to_json_value;
use chrono::Utc;
use serde_json::json;
use std::io::ErrorKind;
use uuid::Uuid;

impl Scheduler {
    pub async fn bootstrap(&self) -> Result<(), SchedulerError> {
        let loaded = self
            .store
            .list_tasks(&TaskListQuery {
                state: None,
                source_kind: None,
            })
            .await?;

        let mut to_resume = Vec::new();
        {
            let mut guard = self.tasks.write().await;
            for task in loaded {
                if matches!(
                    task.state,
                    TaskState::Downloading | TaskState::MetadataFetching | TaskState::Seeding
                ) {
                    to_resume.push(task.id);
                }
                guard.insert(task.id, task);
            }
        }

        for id in to_resume {
            let _ = self.start_task_execution(id).await;
        }
        Ok(())
    }

    pub async fn settings(&self) -> GlobalSettings {
        self.settings.read().await.clone()
    }

    pub async fn update_settings(
        &self,
        settings: GlobalSettings,
    ) -> Result<GlobalSettings, SchedulerError> {
        self.store.upsert_global_settings(&settings).await?;
        let mut guard = self.settings.write().await;
        *guard = settings.clone();
        drop(guard);

        self.emit(None, "settings_updated", to_json_value(&settings));
        Ok(settings)
    }

    pub async fn create_task(&self, mut spec: TaskSpec) -> Result<TaskView, SchedulerError> {
        if spec.settings.is_none() {
            spec.settings = Some(TaskRuntimeSettings::default());
        }
        resolve_source_kind(&mut spec)?;
        {
            let settings = self.settings.read().await;
            resolve_destination_path(&settings.download_dir, &spec.destination_path)
                .map_err(|err| SchedulerError::InvalidDestination(err.to_string()))?;
        }

        let backend_name = self
            .find_backend(&spec)
            .map(|backend| backend.name())
            .ok_or(SchedulerError::UnsupportedSource)?;

        let now = Utc::now();
        let task = TaskView {
            id: Uuid::new_v4(),
            state: TaskState::Queued,
            spec,
            progress: TaskProgress::default(),
            error: None,
            created_at: now,
            updated_at: now,
        };

        self.store.insert_task(&task).await?;
        let mut guard = self.tasks.write().await;
        guard.insert(task.id, task.clone());
        drop(guard);

        self.emit(
            Some(task.id),
            "task_created",
            json!({ "task": task, "backend": backend_name }),
        );
        Ok(task)
    }

    pub async fn list_tasks(&self, query: &TaskListQuery) -> Result<Vec<TaskView>, SchedulerError> {
        self.store
            .list_tasks(query)
            .await
            .map_err(SchedulerError::from)
    }

    pub async fn get_task(&self, id: Uuid) -> Result<TaskView, SchedulerError> {
        self.store.get_task(id).await.map_err(SchedulerError::from)
    }

    pub async fn patch_task(&self, id: Uuid, patch: TaskPatch) -> Result<TaskView, SchedulerError> {
        let mut guard = self.tasks.write().await;
        let task = guard.get_mut(&id).ok_or(SchedulerError::NotFound)?;
        if let Some(settings) = patch.settings {
            task.spec.settings = Some(settings);
        }
        if let Some(concurrency) = patch.concurrency {
            task.spec.concurrency = Some(concurrency.max(1));
        }
        task.updated_at = Utc::now();
        let cloned = task.clone();
        drop(guard);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_settings_updated", to_json_value(&cloned));
        Ok(cloned)
    }

    pub async fn pause_task(&self, id: Uuid) -> Result<TaskView, SchedulerError> {
        self.cancel_task(id).await;
        self.transition(id, TaskState::Paused).await
    }

    pub async fn resume_task(&self, id: Uuid) -> Result<TaskView, SchedulerError> {
        let task = self.transition(id, TaskState::Downloading).await?;
        self.start_task_execution(id).await?;
        Ok(task)
    }

    pub async fn remove_task(&self, id: Uuid, delete_file: bool) -> Result<TaskView, SchedulerError> {
        self.cancel_task(id).await;

        let mut guard = self.tasks.write().await;
        let mut task = guard.remove(&id).ok_or(SchedulerError::NotFound)?;
        validate_transition(&task.state, &TaskState::Removed)?;
        task.state = TaskState::Removed;
        task.progress.download_rate_bps = 0;
        task.progress.upload_rate_bps = 0;
        task.updated_at = Utc::now();
        task.error = None;
        let removed = task.clone();
        drop(guard);

        self.store.delete_task(id).await?;
        self.emit(Some(id), "task_state_changed", to_json_value(&removed));

        if delete_file {
            self.delete_task_file(&removed).await;
        }

        Ok(removed)
    }

    pub async fn verify_task(&self, id: Uuid) -> Result<TaskView, SchedulerError> {
        self.transition(id, TaskState::MetadataFetching).await
    }

    async fn transition(&self, id: Uuid, next: TaskState) -> Result<TaskView, SchedulerError> {
        let mut guard = self.tasks.write().await;
        let task = guard.get_mut(&id).ok_or(SchedulerError::NotFound)?;
        validate_transition(&task.state, &next)?;
        task.state = next.clone();

        if next == TaskState::MetadataFetching {
            task.progress.verified = false;
        }
        if matches!(
            next,
            TaskState::Paused | TaskState::Removed | TaskState::Completed
        ) {
            task.progress.download_rate_bps = 0;
            task.progress.upload_rate_bps = 0;
        }

        task.updated_at = Utc::now();
        task.error = None;
        let cloned = task.clone();
        drop(guard);

        self.store.save_task(&cloned).await?;
        self.emit(Some(id), "task_state_changed", to_json_value(&cloned));
        Ok(cloned)
    }

    async fn delete_task_file(&self, task: &TaskView) {
        let settings = self.settings.read().await;
        let path = match resolve_destination_path(&settings.download_dir, &task.spec.destination_path)
        {
            Ok(path) => path,
            Err(err) => {
                self.emit(
                    Some(task.id),
                    "task_warning",
                    json!({
                        "task_id": task.id,
                        "message": format!("failed to resolve task file path for deletion: {err}"),
                    }),
                );
                return;
            }
        };
        drop(settings);

        let remove_result = if matches!(
            task.spec.source.kind,
            SourceKind::Torrent | SourceKind::Magnet
        ) {
            tokio::fs::remove_dir_all(&path).await
        } else {
            tokio::fs::remove_file(&path).await
        };

        match remove_result {
            Ok(()) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                self.emit(
                    Some(task.id),
                    "task_warning",
                    json!({
                        "task_id": task.id,
                        "message": format!("failed to delete task file {}: {err}", path.display()),
                    }),
                );
            }
        }
    }
}
