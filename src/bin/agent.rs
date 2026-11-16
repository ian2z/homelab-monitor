use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use homelab_monitor::collector::{SharedCollector, SystemCollector};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

async fn get_metrics(State(collector): State<SharedCollector>) -> impl IntoResponse {
    let mut collector = collector.lock().await;
    let metrics = collector.collect();
    Json(metrics)
}

async fn health_check() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(9191);

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("==================================================");
    println!("  Homelab Agent v{}", env!("CARGO_PKG_VERSION"));
    println!("  Listening on: http://{}", addr);
    println!("  Metrics endpoint: http://{}/metrics", addr);
    println!("==================================================");

    let collector = Arc::new(Mutex::new(SystemCollector::new()));

    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .route("/health", get(health_check))
        .with_state(collector);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
