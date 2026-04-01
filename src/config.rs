use crate::constants::{DEFAULT_DB_PATH, DEFAULT_HTTP_CHUNK_SIZE_BYTES};
use crate::models::GlobalSettings;
use std::env;

pub struct BootstrapConfig {
    pub bind_addr: String,
    pub settings: GlobalSettings,
}

impl BootstrapConfig {
    pub fn from_env() -> Self {
        let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
        let db_path = env::var("DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());
        let download_dir =
            env::var("DOWNLOAD_DIR").unwrap_or_else(|_| "data/downloads".to_string());
        let session_dir = env::var("SESSION_DIR").unwrap_or_else(|_| "data/session".to_string());
        let token = env::var("AUTH_TOKEN").unwrap_or_else(|_| "change-me".to_string());
        let http_chunk_size_bytes = env::var("HTTP_CHUNK_SIZE_BYTES")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_HTTP_CHUNK_SIZE_BYTES);

        let settings = GlobalSettings {
            auth_token: token,
            download_dir,
            session_dir,
            db_path,
            global_connection_limit: 2048,
            default_task_connection_limit: 16,
            http_chunk_size_bytes,
            default_seeding_ratio_limit: 2.0,
            default_seeding_time_limit_secs: 86_400,
            default_proxy: None,
        };

        Self {
            bind_addr,
            settings,
        }
    }
}
