use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct WeatherAnalysisQuery {
    pub tower_id: Option<Uuid>,
    pub days: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ForecastQuery {
    pub tower_id: Option<Uuid>,
}

pub async fn analyze_weather_impact(
    State(state): State<AppState>,
    Query(params): Query<WeatherAnalysisQuery>,
) -> Result<Json<weather::WeatherImpactAnalysis>, AppError> {
    let analysis = services::weather_analysis::analyze_weather_impact(
        state,
        params.tower_id,
        params.days.unwrap_or(7),
    ).await?;
    Ok(Json(analysis))
}

pub async fn get_weather_forecast(
    State(state): State<AppState>,
    Query(params): Query<ForecastQuery>,
) -> Result<Json<weather::WeatherForecast>, AppError> {
    let forecast = services::weather_analysis::get_weather_forecast(
        state,
        params.tower_id,
    ).await?;
    Ok(Json(forecast))
}

pub async fn get_weather_data(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    Query(params): Query<super::data::DataQueryParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = services::sensor::get_sensor_data(
        state,
        tower_id,
        "weather",
        params.start_time.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(24)),
        params.end_time.unwrap_or_else(|| Utc::now()),
    ).await?;
    Ok(Json(data))
}
