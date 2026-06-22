use crate::{AppState, AppError, models::*};
use chrono::{Utc};
use uuid::Uuid;
use tracing::{info};

pub async fn create_tower(
    state: AppState,
    req: &tower::CreateTowerRequest,
) -> Result<Uuid, AppError> {
    info!("Creating tower: {}", req.name);

    let tower = tower::Tower {
        id: Uuid::new_v4(),
        name: req.name.clone(),
        code: req.code.clone(),
        location: req.location.clone(),
        height_meters: req.height_meters,
        construction_date: req.construction_date,
        status: req.status.clone(),
        cable_line_id: req.cable_line_id,
        position_in_line: req.position_in_line,
        max_load_kg: req.max_load_kg,
        last_inspection_date: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        sensors: Vec::new(),
    };

    state.clickhouse_client.insert_tower(&tower).await?;

    Ok(tower.id)
}

pub async fn get_tower(
    state: AppState,
    id: Uuid,
) -> Result<tower::Tower, AppError> {
    info!("Getting tower: {}", id);

    state.clickhouse_client.get_tower(id).await?
        .ok_or_else(|| AppError::NotFound(format!("Tower {} not found", id)))
}

pub async fn list_towers(
    state: AppState,
    query: &tower::TowerListQuery,
) -> Result<Vec<tower::Tower>, AppError> {
    info!("Listing towers");

    state.clickhouse_client.list_towers(query).await
}

pub async fn update_tower(
    state: AppState,
    id: Uuid,
    req: &tower::UpdateTowerRequest,
) -> Result<(), AppError> {
    info!("Updating tower: {}", id);

    let existing = get_tower(state.clone(), id).await?;

    let updated = tower::Tower {
        name: req.name.clone().unwrap_or(existing.name),
        status: req.status.clone().unwrap_or(existing.status),
        location: req.location.clone().unwrap_or(existing.location),
        height_meters: req.height_meters.unwrap_or(existing.height_meters),
        max_load_kg: req.max_load_kg.unwrap_or(existing.max_load_kg),
        updated_at: Utc::now(),
        ..existing
    };

    state.clickhouse_client.insert_tower(&updated).await?;

    Ok(())
}

pub async fn delete_tower(
    state: AppState,
    id: Uuid,
) -> Result<(), AppError> {
    info!("Deleting tower: {}", id);
    Ok(())
}

pub async fn get_tower_status(
    state: AppState,
    tower_id: Uuid,
) -> Result<tower::TowerStatusResponse, AppError> {
    info!("Getting tower status: {}", tower_id);

    use chrono::Duration;
    let end_time = Utc::now();
    let start_time = end_time - Duration::minutes(5);

    let ice_data = state.clickhouse_client.query_ice_detection_data(
        &ice_detection::IceQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            page: Some(0),
            page_size: Some(100),
        },
    ).await?;

    let vibration_data = state.clickhouse_client.query_vibration_data(
        &vibration::VibrationQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            frequency_range: None,
            page: Some(0),
            page_size: Some(100),
        },
    ).await?;

    let wind_data = state.clickhouse_client.query_wind_speed_data(
        &vibration::VibrationQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            frequency_range: None,
            page: Some(0),
            page_size: Some(100),
        },
    ).await?;

    let tower = get_tower(state.clone(), tower_id).await?;

    let avg_ice = ice_data.iter().map(|d| d.ice_thickness_mm).sum::<f64>() / ice_data.len().max(1) as f64;
    let avg_vibration = vibration_data.iter().map(|d| d.velocity_mm_s).sum::<f64>() / vibration_data.len().max(1) as f64;
    let avg_wind = wind_data.iter().map(|d| d.velocity_mm_s).sum::<f64>() / wind_data.len().max(1) as f64;

    let risk_level = calculate_risk_level(avg_vibration, avg_wind, avg_ice);

    Ok(tower::TowerStatusResponse {
        tower_id,
        tower_name: tower.name,
        status: tower.status,
        vibration_level: avg_vibration,
        wind_speed: avg_wind,
        ice_thickness: avg_ice,
        risk_level,
        last_update: end_time,
    })
}

pub async fn get_all_towers_status(
    state: AppState,
) -> Result<Vec<tower::TowerStatusResponse>, AppError> {
    info!("Getting all towers status");

    let towers = list_towers(state.clone(), &tower::TowerListQuery {
        cable_line_id: None,
        status: None,
        page: Some(0),
        page_size: Some(100),
    }).await?;

    let mut statuses = Vec::new();

    for tower in &towers {
        match get_tower_status(state.clone(), tower.id).await {
            Ok(status) => statuses.push(status),
            Err(e) => tracing::warn!("Failed to get status for tower {}: {}", tower.id, e),
        }
    }

    Ok(statuses)
}

fn calculate_risk_level(vibration: f64, wind_speed: f64, ice_thickness: f64) -> tower::RiskLevel {
    let mut score = 0.0;

    if vibration > 5.0 { score += 40.0; }
    else if vibration > 3.0 { score += 20.0; }
    else if vibration > 1.5 { score += 10.0; }

    if wind_speed > 25.0 { score += 40.0; }
    else if wind_speed > 17.0 { score += 25.0; }
    else if wind_speed > 10.0 { score += 10.0; }

    if ice_thickness > 10.0 { score += 50.0; }
    else if ice_thickness > 5.0 { score += 30.0; }
    else if ice_thickness > 2.0 { score += 15.0; }

    match score {
        s if s >= 80.0 => tower::RiskLevel::Critical,
        s if s >= 50.0 => tower::RiskLevel::High,
        s if s >= 30.0 => tower::RiskLevel::Medium,
        s if s >= 10.0 => tower::RiskLevel::Low,
        _ => tower::RiskLevel::None,
    }
}
