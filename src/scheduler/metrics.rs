use super::Scheduler;
use crate::constants::APP_NAME;
use crate::models::{StatsView, TaskCounts, TaskState};

impl Scheduler {
    pub async fn stats(&self) -> StatsView {
        let guard = self.tasks.read().await;
        let mut counts = TaskCounts::default();
        let mut active_download_rate_bps = 0_u64;
        let mut active_upload_rate_bps = 0_u64;
        let mut active_tasks = 0_usize;

        for task in guard.values() {
            match task.state {
                TaskState::Queued => counts.queued += 1,
                TaskState::MetadataFetching => counts.metadata_fetching += 1,
                TaskState::Downloading => counts.downloading += 1,
                TaskState::Seeding => counts.seeding += 1,
                TaskState::Paused => counts.paused += 1,
                TaskState::Completed => counts.completed += 1,
                TaskState::Failed => counts.failed += 1,
                TaskState::Removed => counts.removed += 1,
            }
            if matches!(task.state, TaskState::Downloading | TaskState::Seeding) {
                active_tasks += 1;
                active_download_rate_bps =
                    active_download_rate_bps.saturating_add(task.progress.download_rate_bps);
                active_upload_rate_bps =
                    active_upload_rate_bps.saturating_add(task.progress.upload_rate_bps);
            }
        }

        StatsView {
            task_counts: counts.clone(),
            active_download_rate_bps,
            active_upload_rate_bps,
            queued_tasks: counts.queued,
            active_tasks,
        }
    }

    pub async fn metrics_text(&self) -> String {
        let stats = self.stats().await;
        format!(
            concat!(
                "# TYPE {}_tasks_queued gauge\n",
                "{}_tasks_queued {}\n",
                "# TYPE {}_tasks_downloading gauge\n",
                "{}_tasks_downloading {}\n",
                "# TYPE {}_tasks_completed gauge\n",
                "{}_tasks_completed {}\n",
                "# TYPE {}_active_download_rate_bps gauge\n",
                "{}_active_download_rate_bps {}\n",
                "# TYPE {}_active_upload_rate_bps gauge\n",
                "{}_active_upload_rate_bps {}\n"
            ),
            APP_NAME,
            APP_NAME,
            stats.task_counts.queued,
            APP_NAME,
            APP_NAME,
            stats.task_counts.downloading,
            APP_NAME,
            APP_NAME,
            stats.task_counts.completed,
            APP_NAME,
            APP_NAME,
            stats.active_download_rate_bps,
            APP_NAME,
            APP_NAME,
            stats.active_upload_rate_bps,
        )
    }
}
