use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct EvaluateStrategyRequest {
    pub alert_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteShutdownRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StrategyQueryParams {
    pub tower_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn evaluate_shutdown_strategy(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Json(req): Json<EvaluateStrategyRequest>,
) -> Result<Json<shutdown_strategy::ShutdownStrategy>, AppError> {
    let strategy = services::shutdown_strategy::evaluate_shutdown_strategy(
        state,
        tower_id,
        req.alert_id,
    ).await?;
    Ok(Json(strategy))
}

pub async fn execute_shutdown(
    State(state): State<AppState>,
    Path(strategy_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<ExecuteShutdownRequest>,
) -> Result<Json<shutdown_strategy::ShutdownLog>, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let log = services::shutdown_strategy::execute_shutdown(
        state,
        strategy_id,
        user_id,
        req.notes,
    ).await?;
    Ok(Json(log))
}

pub async fn get_active_strategies(
    State(state): State<AppState>,
    Query(params): Query<StrategyQueryParams>,
) -> Result<Json<Vec<shutdown_strategy::ShutdownStrategy>>, AppError> {
    let strategies = services::shutdown_strategy::get_active_strategies(
        state,
        params.tower_id,
    ).await?;
    Ok(Json(strategies))
}

pub async fn get_shutdown_logs(
    State(state): State<AppState>,
    Query(params): Query<StrategyQueryParams>,
) -> Result<Json<Vec<shutdown_strategy::ShutdownLog>>, AppError> {
    let logs = state.clickhouse_client.get_shutdown_logs(
        params.tower_id,
        None,
        None,
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(100),
    ).await?;
    Ok(Json(logs))
}
