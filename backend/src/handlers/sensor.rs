use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct SensorListQueryParams {
    pub tower_id: Option<Uuid>,
    pub sensor_type: Option<String>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn create_sensor(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Json(req): Json<sensor::CreateSensorRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = services::sensor::create_sensor(state, tower_id, &req).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn get_sensor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<sensor::Sensor>, AppError> {
    let sensor = services::sensor::get_sensor(state, id).await?;
    Ok(Json(sensor))
}

pub async fn list_sensors(
    State(state): State<AppState>,
    Query(params): Query<SensorListQueryParams>,
) -> Result<Json<Vec<sensor::Sensor>>, AppError> {
    let sensor_type = params.sensor_type.as_ref().and_then(|s| sensor::SensorType::from_str(s).ok());
    let status = params.status.as_ref().and_then(|s| sensor::SensorStatus::from_str(s).ok());
    let sensors = services::sensor::list_sensors(
        state,
        params.tower_id,
        sensor_type,
        status,
        params.page.unwrap_or(0),
        params.page_size.unwrap_or(100),
    ).await?;
    Ok(Json(sensors))
}

pub async fn update_sensor_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateSensorStatusRequest>,
) -> Result<StatusCode, AppError> {
    let status = sensor::SensorStatus::from_str(&req.status)
        .map_err(|e| AppError::BadRequest(format!("Invalid sensor status: {}", e)))?;
    services::sensor::update_sensor_status(state, id, status).await?;
    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct UpdateSensorStatusRequest {
    pub status: String,
}

pub async fn calibrate_sensor(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    services::sensor::calibrate_sensor(state, id, user_id).await?;
    Ok(StatusCode::OK)
}
