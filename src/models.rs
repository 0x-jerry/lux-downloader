use crate::constants::DEFAULT_HTTP_CHUNK_SIZE_BYTES;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Queued,
    MetadataFetching,
    Downloading,
    Seeding,
    Paused,
    Completed,
    Failed,
    Removed,
}

impl Display for TaskState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::MetadataFetching => "metadata_fetching",
            Self::Downloading => "downloading",
            Self::Seeding => "seeding",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Removed => "removed",
        };
        write!(f, "{value}")
    }
}

impl FromStr for TaskState {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "metadata_fetching" => Ok(Self::MetadataFetching),
            "downloading" => Ok(Self::Downloading),
            "seeding" => Ok(Self::Seeding),
            "paused" => Ok(Self::Paused),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "removed" => Ok(Self::Removed),
            other => Err(format!("unknown task state: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Auto,
    Url,
    Magnet,
    Torrent,
    Metalink,
}

impl Default for SourceKind {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInput {
    #[serde(default)]
    pub kind: SourceKind,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskRuntimeSettings {
    pub priority: Option<i32>,
    pub max_connections: Option<u32>,
    pub max_download_rate_bps: Option<u64>,
    pub max_upload_rate_bps: Option<u64>,
    pub proxy: Option<String>,
    pub headers: Option<Vec<HeaderPair>>,
    pub cookies: Option<Vec<CookiePair>>,
    pub seeding_ratio_limit: Option<f32>,
    pub seeding_time_limit_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookiePair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub source: SourceInput,
    pub destination_path: String,
    #[serde(default)]
    pub overwrite_existing: bool,
    pub concurrency: Option<u32>,
    pub checksum: Option<String>,
    pub protocol_options: Option<Value>,
    pub settings: Option<TaskRuntimeSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub download_rate_bps: u64,
    pub upload_rate_bps: u64,
    pub verified: bool,
}

impl Default for TaskProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            total_bytes: None,
            download_rate_bps: 0,
            upload_rate_bps: 0,
            verified: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskView {
    pub id: Uuid,
    pub state: TaskState,
    pub spec: TaskSpec,
    pub progress: TaskProgress,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    pub auth_token: String,
    pub download_dir: String,
    pub session_dir: String,
    pub db_path: String,
    pub global_connection_limit: u32,
    pub default_task_connection_limit: u32,
    #[serde(default = "default_http_chunk_size_bytes")]
    pub http_chunk_size_bytes: u64,
    pub default_seeding_ratio_limit: f32,
    pub default_seeding_time_limit_secs: u64,
    pub default_proxy: Option<String>,
}

fn default_http_chunk_size_bytes() -> u64 {
    DEFAULT_HTTP_CHUNK_SIZE_BYTES
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub sequence: u64,
    pub task_id: Option<Uuid>,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsView {
    pub task_counts: TaskCounts,
    pub active_download_rate_bps: u64,
    pub active_upload_rate_bps: u64,
    pub queued_tasks: usize,
    pub active_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskCounts {
    pub queued: usize,
    pub metadata_fetching: usize,
    pub downloading: usize,
    pub seeding: usize,
    pub paused: usize,
    pub completed: usize,
    pub failed: usize,
    pub removed: usize,
}

#[derive(Debug, Deserialize)]
pub struct TaskListQuery {
    pub state: Option<String>,
    pub source_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TaskPatch {
    pub settings: Option<TaskRuntimeSettings>,
    pub concurrency: Option<u32>,
    pub source: Option<SourceInput>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveTaskQuery {
    #[serde(default)]
    pub delete_file: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    #[serde(flatten)]
    pub spec: TaskSpec,
    #[serde(default)]
    pub auto_start: bool,
}
