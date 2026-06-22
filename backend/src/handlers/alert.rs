use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    pub tower_id: Option<Uuid>,
    pub alert_type: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    pub notes: String,
}

pub async fn get_alerts(
    State(state): State<AppState>,
    Query(params): Query<AlertQueryParams>,
) -> Result<Json<Vec<alert::Alert>>, AppError> {
    let alert_type = params.alert_type.as_ref().and_then(|s| alert::AlertType::from_str(s).ok());
    let severity = params.severity.as_ref().and_then(|s| alert::AlertSeverity::from_str(s).ok());
    let status = params.status.as_ref().and_then(|s| alert::AlertStatus::from_str(s).ok());

    let query = alert::AlertQuery {
        id: None,
        tower_id: params.tower_id,
        alert_type,
        severity,
        status,
        start_time: params.start_time,
        end_time: params.end_time,
        page: params.page,
        page_size: params.page_size,
    };

    let alerts = state.clickhouse_client.query_alerts(&query).await?;
    Ok(Json(alerts))
}

pub async fn get_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<alert::Alert>, AppError> {
    let mut alerts = state.clickhouse_client.query_alerts(&alert::AlertQuery {
        id: Some(id),
        tower_id: None,
        alert_type: None,
        severity: None,
        status: None,
        start_time: None,
        end_time: None,
        page: Some(0),
        page_size: Some(1),
    }).await?;

    let alert = alerts.pop()
        .ok_or_else(|| AppError::NotFound(format!("Alert {} not found", id)))?;

    Ok(Json(alert))
}

pub async fn acknowledge_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<AcknowledgeAlertRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    services::alert::acknowledge_alert(state, id, user_id, req.notes).await?;
    Ok(StatusCode::OK)
}

pub async fn resolve_alert(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<ResolveAlertRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    services::alert::resolve_alert(state, id, user_id, req.notes).await?;
    Ok(StatusCode::OK)
}

pub async fn get_alert_summary(
    State(state): State<AppState>,
) -> Result<Json<alert::AlertSummary>, AppError> {
    let summary = services::alert::get_alert_summary(state).await?;
    Ok(Json(summary))
}
