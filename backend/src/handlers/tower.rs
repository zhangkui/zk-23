use crate::{AppState, AppError, models::*, services};
use axum::{
    extract::{State, Path, Query, Json},
    http::StatusCode,
};
use uuid::Uuid;
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct TowerListQueryParams {
    pub cable_line_id: Option<Uuid>,
    pub status: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn create_tower(
    State(state): State<AppState>,
    Json(req): Json<tower::CreateTowerRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = services::tower::create_tower(state, &req).await?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn get_tower(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<tower::Tower>, AppError> {
    let tower = services::tower::get_tower(state, id).await?;
    Ok(Json(tower))
}

pub async fn list_towers(
    State(state): State<AppState>,
    Query(params): Query<TowerListQueryParams>,
) -> Result<Json<Vec<tower::Tower>>, AppError> {
    let status = params.status.as_ref().and_then(|s| tower::TowerStatus::from_str(s).ok());
    let query = tower::TowerListQuery {
        cable_line_id: params.cable_line_id,
        status,
        page: params.page,
        page_size: params.page_size,
    };
    let towers = services::tower::list_towers(state, &query).await?;
    Ok(Json(towers))
}

pub async fn update_tower(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<tower::UpdateTowerRequest>,
) -> Result<StatusCode, AppError> {
    services::tower::update_tower(state, id, &req).await?;
    Ok(StatusCode::OK)
}

pub async fn delete_tower(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    services::tower::delete_tower(state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_tower_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<tower::TowerStatusResponse>, AppError> {
    let status = services::tower::get_tower_status(state, id).await?;
    Ok(Json(status))
}

pub async fn get_all_towers_status(
    State(state): State<AppState>,
) -> Result<Json<Vec<tower::TowerStatusResponse>>, AppError> {
    let statuses = services::tower::get_all_towers_status(state).await?;
    Ok(Json(statuses))
}
