use crate::{AppState, AppError, models::*, mq, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct DataQueryParams {
    pub data_type: Option<String>,
    pub start_time: Option<chrono::DateTime<Utc>>,
    pub end_time: Option<chrono::DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn get_sensor_data(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<DataQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data_type = params.data_type.unwrap_or_else(|| "vibration".to_string());
    let data = services::sensor::get_sensor_data(
        state,
        tower_id,
        &data_type,
        params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1)),
        params.end_time.unwrap_or_else(|| Utc::now()),
    ).await?;
    Ok(Json(data))
}

pub async fn get_vibration_data(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<DataQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = services::sensor::get_sensor_data(
        state,
        tower_id,
        "vibration",
        params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1)),
        params.end_time.unwrap_or_else(|| Utc::now()),
    ).await?;
    Ok(Json(data))
}

pub async fn get_wind_data(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<DataQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = services::sensor::get_sensor_data(
        state,
        tower_id,
        "wind_speed",
        params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1)),
        params.end_time.unwrap_or_else(|| Utc::now()),
    ).await?;
    Ok(Json(data))
}

pub async fn get_ice_data(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<DataQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = services::sensor::get_sensor_data(
        state,
        tower_id,
        "ice_detection",
        params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1)),
        params.end_time.unwrap_or_else(|| Utc::now()),
    ).await?;
    Ok(Json(data))
}

pub async fn ingest_sensor_data(
    State(state): State<AppState>,
    Path((tower_id, data_type)): Path<(Uuid, String)>,
    Json(data): Json<serde_json::Value>,
) -> Result<StatusCode, AppError> {
    use sensor::ReadingQuality;

    let sensor_id = Uuid::new_v4();
    let now = Utc::now();

    match data_type.as_str() {
        "vibration" => {
            let reading: vibration::VibrationReading = serde_json::from_value(data)?;
            mq::publisher::publish_vibration_reading(&state, &reading).await?;
        }
        "wind_speed" => {
            let reading: wind_speed::WindSpeedReading = serde_json::from_value(data)?;
            mq::publisher::publish_wind_speed_reading(&state, &reading).await?;
        }
        "ice_detection" => {
            let reading: ice_detection::IceReading = serde_json::from_value(data)?;
            mq::publisher::publish_ice_detection_reading(&state, &reading).await?;
        }
        _ => {
            return Err(AppError::BadRequest(format!("Unknown data type: {}", data_type)));
        }
    }

    Ok(StatusCode::ACCEPTED)
}
