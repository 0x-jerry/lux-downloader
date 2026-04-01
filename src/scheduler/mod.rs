mod error;
mod execution;
mod lifecycle;
mod metrics;
mod source;
mod transitions;

use crate::backends::{TransferBackend, default_backends};
use crate::models::{EventEnvelope, GlobalSettings, TaskSpec, TaskView};
use crate::persistence::Store;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{RwLock, broadcast};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub use error::SchedulerError;

#[derive(Clone)]
pub struct Scheduler {
    store: Store,
    tasks: Arc<RwLock<HashMap<Uuid, TaskView>>>,
    sequence: Arc<AtomicU64>,
    events: broadcast::Sender<EventEnvelope>,
    settings: Arc<RwLock<GlobalSettings>>,
    backends: Arc<Vec<Arc<dyn TransferBackend>>>,
    active: Arc<RwLock<HashMap<Uuid, CancellationToken>>>,
}

impl Scheduler {
    pub fn new(store: Store, settings: GlobalSettings) -> Self {
        let (events, _) = broadcast::channel(1024);
        Self {
            store,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            sequence: Arc::new(AtomicU64::new(0)),
            events,
            settings: Arc::new(RwLock::new(settings)),
            backends: Arc::new(default_backends()),
            active: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope> {
        self.events.subscribe()
    }

    fn emit(&self, task_id: Option<Uuid>, event_type: &str, payload: serde_json::Value) {
        let envelope = EventEnvelope {
            sequence: self.sequence.fetch_add(1, Ordering::Relaxed) + 1,
            task_id,
            event_type: event_type.to_string(),
            timestamp: Utc::now(),
            payload,
        };
        let _ = self.events.send(envelope);
    }

    fn find_backend(&self, spec: &TaskSpec) -> Option<Arc<dyn TransferBackend>> {
        self.backends
            .iter()
            .find(|backend| backend.can_handle(spec))
            .cloned()
    }
}
