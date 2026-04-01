use crate::models::{GlobalSettings, TaskListQuery, TaskProgress, TaskState, TaskView};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("not found")]
    NotFound,
    #[error("invalid state in db: {0}")]
    InvalidState(String),
    #[error("invalid time format: {0}")]
    InvalidTime(String),
}

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn connect(db_path: &str) -> Result<Self, StoreError> {
        let conn = format!("sqlite://{db_path}?mode=rwc");
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&conn)
            .await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<(), StoreError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL,
                spec_json TEXT NOT NULL,
                progress_json TEXT NOT NULL,
                error TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS global_settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                data_json TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_global_settings(
        &self,
        settings: &GlobalSettings,
    ) -> Result<(), StoreError> {
        let payload = serde_json::to_string(settings)?;
        sqlx::query(
            r#"
            INSERT INTO global_settings (id, data_json)
            VALUES (1, ?)
            ON CONFLICT(id) DO UPDATE SET data_json = excluded.data_json
            "#,
        )
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn load_global_settings(&self) -> Result<Option<GlobalSettings>, StoreError> {
        let row = sqlx::query("SELECT data_json FROM global_settings WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => {
                let value: String = r.get("data_json");
                Ok(Some(serde_json::from_str(&value)?))
            }
            None => Ok(None),
        }
    }

    pub async fn insert_task(&self, task: &TaskView) -> Result<(), StoreError> {
        self.save_task(task).await
    }

    pub async fn save_task(&self, task: &TaskView) -> Result<(), StoreError> {
        let spec_json = serde_json::to_string(&task.spec)?;
        let progress_json = serde_json::to_string(&task.progress)?;
        sqlx::query(
            r#"
            INSERT INTO tasks (id, state, spec_json, progress_json, error, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id)
            DO UPDATE SET
                state = excluded.state,
                spec_json = excluded.spec_json,
                progress_json = excluded.progress_json,
                error = excluded.error,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(task.id.to_string())
        .bind(task.state.to_string())
        .bind(spec_json)
        .bind(progress_json)
        .bind(task.error.as_ref())
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_task(&self, id: Uuid) -> Result<TaskView, StoreError> {
        let row = sqlx::query(
            "SELECT id, state, spec_json, progress_json, error, created_at, updated_at FROM tasks WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        row.map(Self::task_from_row)
            .transpose()?
            .ok_or(StoreError::NotFound)
    }

    pub async fn delete_task(&self, id: Uuid) -> Result<(), StoreError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StoreError::NotFound);
        }
        Ok(())
    }

    pub async fn list_tasks(&self, query: &TaskListQuery) -> Result<Vec<TaskView>, StoreError> {
        let rows = sqlx::query(
            "SELECT id, state, spec_json, progress_json, error, created_at, updated_at FROM tasks",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let task = Self::task_from_row(row)?;
            if let Some(state) = query.state.as_ref() {
                if task.state.to_string() != *state {
                    continue;
                }
            }
            if let Some(kind) = query.source_kind.as_ref() {
                let kind_matches = serde_json::to_value(&task.spec.source.kind)?
                    .as_str()
                    .map(|v| v == kind)
                    .unwrap_or(false);
                if !kind_matches {
                    continue;
                }
            }
            items.push(task);
        }
        Ok(items)
    }

    fn task_from_row(row: sqlx::sqlite::SqliteRow) -> Result<TaskView, StoreError> {
        let state_str: String = row.get("state");
        let id_str: String = row.get("id");
        let spec_json: String = row.get("spec_json");
        let progress_json: String = row.get("progress_json");
        let created_at: String = row.get("created_at");
        let updated_at: String = row.get("updated_at");

        let id = Uuid::parse_str(&id_str).map_err(|_| StoreError::NotFound)?;
        let state = TaskState::from_str(&state_str).map_err(StoreError::InvalidState)?;
        let spec = serde_json::from_str(&spec_json)?;
        let progress: TaskProgress = serde_json::from_str(&progress_json)?;
        let created_at = parse_time(&created_at)?;
        let updated_at = parse_time(&updated_at)?;
        let error = row.get("error");

        Ok(TaskView {
            id,
            state,
            spec,
            progress,
            error,
            created_at,
            updated_at,
        })
    }
}

fn parse_time(v: &str) -> Result<DateTime<Utc>, StoreError> {
    DateTime::parse_from_rfc3339(v)
        .map(|ts| ts.with_timezone(&Utc))
        .map_err(|_| StoreError::InvalidTime(v.to_string()))
}

pub fn to_json_value<T: Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value)
        .unwrap_or_else(|_| serde_json::json!({ "error": "serialization_failed" }))
}
