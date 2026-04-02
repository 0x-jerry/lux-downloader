pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const DEFAULT_DB_PATH: &str = concat!("data/", env!("CARGO_PKG_NAME"), ".db");
pub const DEFAULT_HTTP_CHUNK_SIZE_BYTES: u64 = 2 * 1024 * 1024;
