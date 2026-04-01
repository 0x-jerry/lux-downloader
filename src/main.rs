mod api;
mod auth;
mod backends;
mod config;
mod constants;
mod models;
mod persistence;
mod scheduler;

use crate::config::BootstrapConfig;
use crate::constants::APP_NAME;
use crate::models::GlobalSettings;
use crate::persistence::Store;
use crate::scheduler::Scheduler;
use axum::Router;
use axum::middleware;
use axum::routing::{get, post};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[derive(Clone)]
pub struct AppState {
    scheduler: Arc<Scheduler>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let bootstrap = BootstrapConfig::from_env();
    ensure_parent_dirs(&bootstrap.settings);

    let store = Store::connect(&bootstrap.settings.db_path).await?;
    store.init().await?;

    let settings = match store.load_global_settings().await? {
        Some(value) => value,
        None => {
            store.upsert_global_settings(&bootstrap.settings).await?;
            bootstrap.settings.clone()
        }
    };

    if settings.auth_token == "change-me" {
        warn!("AUTH_TOKEN is default value 'change-me'; set AUTH_TOKEN in production");
    }

    let scheduler = Arc::new(Scheduler::new(store, settings));
    scheduler.bootstrap().await?;

    let state = AppState { scheduler };

    let protected_routes = Router::new()
        .route("/tasks", post(api::create_task).get(api::list_tasks))
        .route("/tasks/{id}", get(api::get_task).patch(api::patch_task))
        .route("/tasks/{id}/pause", post(api::pause_task))
        .route("/tasks/{id}/resume", post(api::resume_task))
        .route("/tasks/{id}/remove", post(api::remove_task))
        .route("/tasks/{id}/verify", post(api::verify_task))
        .route(
            "/settings",
            get(api::get_settings).patch(api::patch_settings),
        )
        .route("/stats", get(api::stats))
        .route("/events", get(api::events_ws))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/metrics", get(api::metrics))
        .merge(protected_routes)
        .with_state(state)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let addr: SocketAddr = bootstrap.bind_addr.parse()?;
    let listener = TcpListener::bind(addr).await?;

    info!("{} listening on {}", APP_NAME, addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn init_tracing() {
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();
}

fn ensure_parent_dirs(settings: &GlobalSettings) {
    for path in [
        &settings.download_dir,
        &settings.session_dir,
        &settings.db_path,
    ] {
        let p = Path::new(path);
        if path == &settings.db_path {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            continue;
        }
        let _ = std::fs::create_dir_all(p);
    }
}
