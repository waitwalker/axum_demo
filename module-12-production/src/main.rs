use axum::{extract::State, routing::get, Json, Router};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
struct AppState {
    ready: Arc<AtomicBool>,
    request_count: Arc<AtomicU64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            ready: Arc::new(AtomicBool::new(true)),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }
}

async fn health() -> &'static str {
    "OK"
}

async fn ready(
    State(state): State<AppState>,
) -> Result<&'static str, (axum::http::StatusCode, &'static str)> {
    if state.ready.load(Ordering::SeqCst) {
        Ok("ready")
    } else {
        Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "not ready"))
    }
}

fn main() {
    println!("Hello, world!");
}
