use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct IceAnalysisQueryParams {
    pub tower_id: Option<Uuid>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn analyze_ice_risk(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
) -> Result<Json<ice_detection::IceAnalysisResult>, AppError> {
    let result = services::ice_risk::analyze_ice_risk(state, tower_id).await?;
    Ok(Json(result))
}

pub async fn get_ice_analysis_history(
    State(state): State<AppState>,
    Query(params): Query<IceAnalysisQueryParams>,
) -> Result<Json<Vec<ice_detection::IceAnalysisResult>>, AppError> {
    let results = state.clickhouse_client.get_ice_analysis_results(
        params.tower_id,
        params.start_time,
        params.end_time,
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(100),
    ).await?;
    Ok(Json(results))
}

pub async fn get_latest_ice_analysis(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
) -> Result<Json<ice_detection::IceAnalysisResult>, AppError> {
    let mut results = state.clickhouse_client.get_ice_analysis_results(
        Some(tower_id),
        None,
        None,
        0,
        1,
    ).await?;

    let result = results.pop()
        .ok_or_else(|| AppError::NotFound(format!("No ice analysis found for tower {}", tower_id)))?;

    Ok(Json(result))
}
