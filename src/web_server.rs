use crate::db::{DatabaseManager, SnapshotRecord, ThresholdConfig};
use crate::ssd_analyzer::SsdAnalyzer;
use crate::telemetry::TelemetryCollector;
use crate::threshold_engine::ThresholdEngine;
use axum::{
    extract::State,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::time::Duration;

#[derive(RustEmbed)]
#[folder = "static/"]
struct Assets;

#[derive(Clone)]
pub struct AppState {
    pub telemetry: Arc<TelemetryCollector>,
    pub db: DatabaseManager,
    pub threshold_engine: Arc<ThresholdEngine>,
}

pub async fn start_server(state: AppState, port: u16) {
    // Background threshold evaluation task
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            let metrics = state_clone.telemetry.collect();
            if let Ok(config) = state_clone.db.get_config() {
                state_clone.threshold_engine.evaluate(&metrics, &config);
            }
        }
    });

    let app = Router::new()
        .route("/api/telemetry", get(get_telemetry))
        .route("/api/ssd", get(get_ssd))
        .route("/api/history", get(get_history))
        .route("/api/snapshot", post(create_manual_snapshot))
        .route("/api/thresholds", get(get_thresholds).post(save_thresholds))
        .fallback(static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("⚡ Matrix1 SystemPulse iniciado en http://127.0.0.1:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_telemetry(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.telemetry.collect();
    Json(metrics)
}

async fn get_ssd() -> impl IntoResponse {
    let reports = SsdAnalyzer::analyze_all();
    Json(reports)
}

async fn get_history(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_snapshots(50) {
        Ok(list) => Json(list).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error DB: {}", err),
        )
            .into_response(),
    }
}

async fn create_manual_snapshot(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.telemetry.collect();
    let top_proc = metrics
        .top_cpu_processes
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let metrics_json = serde_json::to_string(&metrics).unwrap_or_default();

    let record = SnapshotRecord {
        id: None,
        timestamp: metrics.timestamp.clone(),
        trigger_type: "MANUAL".to_string(),
        cpu_usage: metrics.cpu_usage_total,
        memory_percent: metrics.memory_percent,
        status_level: metrics.status_level.clone(),
        top_process_name: top_proc,
        metrics_json,
    };

    match state.db.save_snapshot(&record) {
        Ok(id) => (StatusCode::OK, Json(serde_json::json!({ "id": id, "status": "ok" }))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error guardando snapshot: {}", err),
        )
            .into_response(),
    }
}

async fn get_thresholds(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_config() {
        Ok(cfg) => Json(cfg).into_response(),
        Err(_) => Json(ThresholdConfig::default()).into_response(),
    }
}

async fn save_thresholds(
    State(state): State<AppState>,
    Json(payload): Json<ThresholdConfig>,
) -> impl IntoResponse {
    match state.db.update_config(&payload) {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error actualizando umbrales: {}", err),
        )
            .into_response(),
    }
}

// Embedded Static Files Handler
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => match Assets::get("index.html") {
            Some(index_content) => Html(index_content.data.into_owned()).into_response(),
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        },
    }
}
