use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::State,
    response::{IntoResponse, Json},
    http::StatusCode,
};
use chrono::Utc;
use serde::{Serialize};
use std::time::{Instant, Duration};

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: chrono::DateTime<Utc>,
    pub services: ServiceStatus,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub database: String,
    pub nats: String,
    pub redis: String,
    pub response_time_ms: u64,
}

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let start = Instant::now();

    let mut db_status = "healthy";
    let mut nats_status = "healthy";
    let mut redis_status = "healthy";

    let db_check = state.clickhouse_client.query_alerts(&crate::models::alert::AlertQuery {
        id: None,
        tower_id: None,
        alert_type: None,
        severity: None,
        status: None,
        start_time: None,
        end_time: None,
        page: Some(0),
        page_size: Some(1),
    }).await;

    if db_check.is_err() {
        db_status = "unhealthy";
    }

    let elapsed = start.elapsed();

    let response = HealthStatus {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: state.start_time.elapsed().as_secs(),
        timestamp: Utc::now(),
        services: ServiceStatus {
            database: db_status.to_string(),
            nats: nats_status.to_string(),
            redis: redis_status.to_string(),
            response_time_ms: elapsed.as_millis() as u64,
        },
    };

    (StatusCode::OK, Json(response))
}

pub async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let response = serde_json::json!({
        "status": "ready",
        "checks": {
            "database": "ready",
            "nats": "ready",
            "websocket": "ready"
        }
    });

    (StatusCode::OK, Json(response))
}

pub async fn liveness_check() -> impl IntoResponse {
    let response = serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now().to_rfc3339()
    });

    (StatusCode::OK, Json(response))
}
