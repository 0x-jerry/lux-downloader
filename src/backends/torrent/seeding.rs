use super::BackendContext;
use crate::models::TaskSpec;
use std::time::Duration;
use tokio::time::Instant;

pub(super) const BRIEF_SEEDING_CAP_SECS: u64 = 300;

pub(super) fn effective_ratio_limit(spec: &TaskSpec, context: &BackendContext) -> Option<f32> {
    spec.settings
        .as_ref()
        .and_then(|settings| settings.seeding_ratio_limit)
        .or(Some(context.default_seeding_ratio_limit))
        .filter(|limit| *limit > 0.0)
}

pub(super) fn effective_brief_seeding_secs(spec: &TaskSpec, context: &BackendContext) -> u64 {
    spec.settings
        .as_ref()
        .and_then(|settings| settings.seeding_time_limit_secs)
        .unwrap_or(context.default_seeding_time_limit_secs)
        .min(BRIEF_SEEDING_CAP_SECS)
}

pub(super) fn should_finish_seeding(
    seeding_since: Option<Instant>,
    seeding_secs: u64,
    ratio_limit: Option<f32>,
    uploaded_bytes: u64,
    total_bytes: u64,
) -> bool {
    let elapsed_done = seeding_since
        .map(|since| since.elapsed() >= Duration::from_secs(seeding_secs))
        .unwrap_or(false);

    if elapsed_done {
        return true;
    }

    let Some(limit) = ratio_limit else {
        return false;
    };

    if total_bytes == 0 {
        return false;
    }

    (uploaded_bytes as f64 / total_bytes as f64) >= limit as f64
}

pub(super) fn mibps_to_bps(mibps: f64) -> u64 {
    if !mibps.is_finite() || mibps <= 0.0 {
        return 0;
    }

    (mibps * 1024.0 * 1024.0) as u64
}

#[cfg(test)]
mod tests {
    use super::{BRIEF_SEEDING_CAP_SECS, should_finish_seeding};
    use std::time::Duration;
    use tokio::time::Instant;

    #[test]
    fn completes_when_ratio_reached() {
        let start = Instant::now();
        let finished = should_finish_seeding(Some(start), BRIEF_SEEDING_CAP_SECS, Some(1.0), 100, 100);
        assert!(finished);
    }

    #[test]
    fn completes_when_time_reached() {
        let start = Instant::now() - Duration::from_secs(10);
        let finished = should_finish_seeding(Some(start), 5, None, 0, 100);
        assert!(finished);
    }

    #[test]
    fn keeps_seeding_when_no_limit_reached() {
        let start = Instant::now();
        let finished = should_finish_seeding(Some(start), 300, Some(2.0), 100, 100);
        assert!(!finished);
    }
}
