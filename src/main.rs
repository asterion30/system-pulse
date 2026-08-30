use std::sync::Arc;
use system_pulse::db::DatabaseManager;
use system_pulse::telemetry::TelemetryCollector;
use system_pulse::threshold_engine::ThresholdEngine;
use system_pulse::web_server::{start_server, AppState};

#[tokio::main]
async fn main() {
    println!("--------------------------------------------------");
    println!("⚡ MATRIX1 SYSTEM PULSE - Monitoreo & EOL SSD");
    println!("--------------------------------------------------");

    let db = DatabaseManager::new("system_pulse.db").expect("Fallo al inicializar base de datos SQLite");
    let telemetry = Arc::new(TelemetryCollector::new());
    let threshold_engine = Arc::new(ThresholdEngine::new(db.clone()));

    let state = AppState {
        telemetry,
        db,
        threshold_engine,
    };

    let port = 9090;
    start_server(state, port).await;
}
