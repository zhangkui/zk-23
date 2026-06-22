use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};
use chrono::Utc;

#[derive(Debug, Deserialize)]
pub struct VideoVerificationRequest {
    pub alert_id: Uuid,
    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct ManualVerifyRequest {
    pub ice_confirmed: bool,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct CameraQueryParams {
    pub tower_id: Option<Uuid>,
}

pub async fn request_video_verification(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<VideoVerificationRequest>,
) -> Result<Json<video::VideoVerificationResult>, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let priority = video::VerificationPriority::from_str(&req.priority)
        .map_err(|e| AppError::BadRequest(format!("Invalid priority: {}", e)))?;

    let result = services::video::request_video_verification(
        state,
        tower_id,
        req.alert_id,
        user_id,
        priority,
    ).await?;

    Ok(Json(result))
}

pub async fn start_live_stream(
    State(state): State<AppState>,
    Path(tower_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
) -> Result<Json<video::LiveStreamSession>, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let session = services::video::start_live_stream(state, tower_id, user_id).await?;
    Ok(Json(session))
}

pub async fn stop_live_stream(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
) -> Result<StatusCode, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    services::video::stop_live_stream(state, session_id, user_id).await?;
    Ok(StatusCode::OK)
}

pub async fn manual_verify(
    State(state): State<AppState>,
    Path(result_id): Path<Uuid>,
    claims: crate::models::user::JwtClaims,
    Json(req): Json<ManualVerifyRequest>,
) -> Result<Json<video::VideoVerificationResult>, AppError> {
    let user_id = services::auth::extract_user_id_from_claims(&claims)?;
    let result = services::video::manual_verify(
        state,
        result_id,
        user_id,
        req.ice_confirmed,
        req.notes,
    ).await?;
    Ok(Json(result))
}

pub async fn get_cameras(
    State(state): State<AppState>,
    Query(params): Query<CameraQueryParams>,
) -> Result<Json<Vec<video::Camera>>, AppError> {
    let cameras = services::video::get_camera_list(state, params.tower_id).await?;
    Ok(Json(cameras))
}
