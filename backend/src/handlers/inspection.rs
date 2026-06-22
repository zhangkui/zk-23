use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct InspectionQueryParams {
    pub tower_id: Option<Uuid>,
    pub inspection_type: Option<String>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct InspectionReportQuery {
    pub start_time: chrono::DateTime<Utc>,
    pub end_time: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MaintenanceTaskQuery {
    pub tower_id: Option<Uuid>,
    pub status: Option<String>,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusRequest {
    pub status: String,
    pub notes: Option<String>,
}

pub async fn create_inspection(
    State(state): State<AppState>,
    claims: crate::models::user::JwtClaims,
    Json(record): Json<inspection::InspectionRecord>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let mut record = record;
    record.inspector_id = user_id;
    let id = services::inspection::create_inspection_record(state, &record).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn get_inspections(
    State(state): State<AppState>,
    Query(params): Query<InspectionQueryParams>,
) -> Result<Json<Vec<inspection::InspectionRecord>>, AppError> {
    let inspection_type = params.inspection_type.as_ref().and_then(|s| inspection::InspectionType::from_str(s).ok());
    let records = services::inspection::get_inspection_records(
        state,
        params.tower_id,
        inspection_type,
        params.start_time,
        params.end_time,
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(100),
    ).await?;
    Ok(Json(records))
}

pub async fn get_inspection(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<inspection::InspectionRecord>, AppError> {
    let record = services::inspection::get_inspection_by_id(state, id).await?;
    Ok(Json(record))
}

pub async fn create_maintenance_task(
    State(state): State<AppState>,
    claims: crate::models::user::JwtClaims,
    Json(task): Json<inspection::MaintenanceTask>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = services::inspection::create_maintenance_task(state, &task).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn get_maintenance_tasks(
    State(state): State<AppState>,
    Query(params): Query<MaintenanceTaskQuery>,
) -> Result<Json<Vec<inspection::MaintenanceTask>>, AppError> {
    let status = params.status.as_ref().and_then(|s| inspection::MaintenanceStatus::from_str(s).ok());
    let priority = params.priority.as_ref().and_then(|s| inspection::MaintenancePriority::from_str(s).ok());
    let tasks = services::inspection::get_maintenance_tasks(
        state,
        params.tower_id,
        status,
        priority,
    ).await?;
    Ok(Json(tasks))
}

pub async fn update_maintenance_task_status(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<UpdateTaskStatusRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let status = inspection::MaintenanceStatus::from_str(&req.status)
        .map_err(|e| AppError::BadRequest(format!("Invalid status: {}", e)))?;
    services::inspection::update_maintenance_task_status(
        state,
        task_id,
        status,
        user_id,
        req.notes,
    ).await?;
    Ok(StatusCode::OK)
}

pub async fn generate_inspection_report(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<InspectionReportQuery>,
) -> Result<Json<inspection::InspectionReport>, AppError> {
    let report = services::inspection::generate_inspection_report(
        state,
        tower_id,
        params.start_time,
        params.end_time,
    ).await?;
    Ok(Json(report))
}
