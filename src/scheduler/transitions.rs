use super::SchedulerError;
use crate::models::TaskState;

pub(super) fn validate_transition(
    current: &TaskState,
    next: &TaskState,
) -> Result<(), SchedulerError> {
    if current == next {
        return Ok(());
    }

    let allowed = match current {
        TaskState::Queued => matches!(
            next,
            TaskState::Downloading | TaskState::Removed | TaskState::Paused
        ),
        TaskState::MetadataFetching => matches!(
            next,
            TaskState::Downloading | TaskState::Failed | TaskState::Paused
        ),
        TaskState::Downloading => matches!(
            next,
            TaskState::Paused | TaskState::Completed | TaskState::Failed | TaskState::Removed
        ),
        TaskState::Seeding => matches!(
            next,
            TaskState::Paused | TaskState::Completed | TaskState::Removed
        ),
        TaskState::Paused => matches!(next, TaskState::Downloading | TaskState::Removed),
        TaskState::Completed => matches!(next, TaskState::MetadataFetching | TaskState::Removed),
        TaskState::Failed => matches!(
            next,
            TaskState::Queued | TaskState::Downloading | TaskState::Removed
        ),
        TaskState::Removed => false,
    };

    if allowed {
        Ok(())
    } else {
        Err(SchedulerError::InvalidTransition {
            from: current.to_string(),
            to: next.to_string(),
        })
    }
}
