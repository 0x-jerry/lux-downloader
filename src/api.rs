use crate::AppState;
use crate::backends::BackendError;
use crate::models::{CreateTaskRequest, GlobalSettings, RemoveTaskQuery, TaskListQuery, TaskPatch};
use crate::scheduler::SchedulerError;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, response::Response};
use futures_util::StreamExt;
use serde_json::json;
use uuid::Uuid;

pub async fn health() -> impl IntoResponse {
    Json(json!({ "ok": true }))
}

pub async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let mut task = state.scheduler.create_task(payload.spec).await?;
    if payload.auto_start {
        task = state.scheduler.resume_task(task.id).await?;
    }
    Ok((StatusCode::CREATED, Json(json!(task))))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskListQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let tasks = state.scheduler.list_tasks(&query).await?;
    Ok(Json(json!({ "items": tasks })))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.get_task(id).await?;
    Ok(Json(json!(task)))
}

pub async fn pause_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.pause_task(id).await?;
    Ok(Json(json!(task)))
}

pub async fn resume_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.resume_task(id).await?;
    Ok(Json(json!(task)))
}

pub async fn remove_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<RemoveTaskQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.remove_task(id, query.delete_file).await?;
    Ok(Json(json!(task)))
}

pub async fn verify_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.verify_task(id).await?;
    Ok(Json(json!(task)))
}

pub async fn patch_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(patch): Json<TaskPatch>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let task = state.scheduler.patch_task(id, patch).await?;
    Ok(Json(json!(task)))
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let settings = state.scheduler.settings().await;
    Ok(Json(json!(settings)))
}

pub async fn patch_settings(
    State(state): State<AppState>,
    Json(settings): Json<GlobalSettings>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let updated = state.scheduler.update_settings(settings).await?;
    Ok(Json(json!(updated)))
}

pub async fn stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let stats = state.scheduler.stats().await;
    Ok(Json(json!(stats)))
}

pub async fn torrent_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let stats = state.scheduler.torrent_stats(id).await?;
    Ok(Json(stats))
}

pub async fn metrics(State(state): State<AppState>) -> Result<Response, ApiError> {
    let text = state.scheduler.metrics_text().await;
    Ok(([("content-type", "text/plain; version=0.0.4")], text).into_response())
}

pub async fn events_ws(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_ws(state, socket)))
}

async fn handle_ws(state: AppState, mut socket: WebSocket) {
    let mut receiver = state.scheduler.subscribe_events();
    loop {
        tokio::select! {
            event = receiver.recv() => {
                match event {
                    Ok(event) => {
                        if socket
                            .send(Message::Text(match serde_json::to_string(&event) {
                                Ok(msg) => msg.into(),
                                Err(_) => continue,
                            }))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            incoming = socket.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<SchedulerError> for ApiError {
    fn from(value: SchedulerError) -> Self {
        match value {
            SchedulerError::NotFound => Self::NotFound,
            SchedulerError::UnsupportedSource => {
                Self::BadRequest("unsupported source for available backends".to_string())
            }
            SchedulerError::AlreadyRunning => {
                Self::BadRequest("task is already running".to_string())
            }
            SchedulerError::InvalidDestination(message) => Self::BadRequest(message),
            SchedulerError::InvalidTransition { from, to } => {
                Self::BadRequest(format!("invalid transition from {from} to {to}"))
            }
            SchedulerError::Backend(err) => match err {
                BackendError::Unsupported(_) => Self::BadRequest(err.to_string()),
                _ => Self::Internal(err.to_string()),
            },
            SchedulerError::Store(err) => Self::Internal(err.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
