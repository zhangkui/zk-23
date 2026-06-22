use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};

pub async fn create_alert(
    state: AppState,
    alert: &alert::AlertMessage,
) -> Result<Uuid, AppError> {
    let db_alert = alert::Alert {
        id: alert.id,
        alert_type: alert.alert_type.clone(),
        severity: alert.severity.clone(),
        tower_id: alert.tower_id,
        sensor_id: alert.sensor_id,
        title: alert.title.clone(),
        message: alert.message.clone(),
        timestamp: alert.timestamp,
        data: alert.data.clone(),
        status: alert::AlertStatus::Active,
        acknowledged: alert.acknowledged,
        acknowledged_by: alert.acknowledged_by,
        acknowledged_at: alert.acknowledged_at,
        resolved: false,
        resolved_by: None,
        resolved_at: None,
        resolution_notes: None,
        source_system: "ICE-MONITOR".to_string(),
        correlation_id: Some(Uuid::new_v4()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state.clickhouse_client.insert_alert(&db_alert).await?;

    warn!("Alert created: {:?} - {} - {}", alert.severity, alert.title, alert.message);

    Ok(alert.id)
}

pub async fn acknowledge_alert(
    state: AppState,
    alert_id: Uuid,
    user_id: Uuid,
    notes: Option<String>,
) -> Result<(), AppError> {
    let mut alerts = state.clickhouse_client.query_alerts(&alert::AlertQuery {
        id: Some(alert_id),
        tower_id: None,
        alert_type: None,
        severity: None,
        status: None,
        start_time: None,
        end_time: None,
        page: Some(0),
        page_size: Some(1),
    }).await?;

    let alert = alerts.pop().ok_or_else(|| AppError::NotFound(format!("Alert {} not found", alert_id)))?;

    let updated_alert = alert::Alert {
        acknowledged: true,
        acknowledged_by: Some(user_id),
        acknowledged_at: Some(Utc::now()),
        updated_at: Utc::now(),
        ..alert
    };

    state.clickhouse_client.insert_alert(&updated_alert).await?;

    info!("Alert {} acknowledged by user {}", alert_id, user_id);

    Ok(())
}

pub async fn resolve_alert(
    state: AppState,
    alert_id: Uuid,
    user_id: Uuid,
    notes: String,
) -> Result<(), AppError> {
    let mut alerts = state.clickhouse_client.query_alerts(&alert::AlertQuery {
        id: Some(alert_id),
        tower_id: None,
        alert_type: None,
        severity: None,
        status: None,
        start_time: None,
        end_time: None,
        page: Some(0),
        page_size: Some(1),
    }).await?;

    let alert = alerts.pop().ok_or_else(|| AppError::NotFound(format!("Alert {} not found", alert_id)))?;

    let updated_alert = alert::Alert {
        status: alert::AlertStatus::Resolved,
        resolved: true,
        resolved_by: Some(user_id),
        resolved_at: Some(Utc::now()),
        resolution_notes: Some(notes),
        updated_at: Utc::now(),
        acknowledged: true,
        acknowledged_by: alert.acknowledged_by.or(Some(user_id)),
        acknowledged_at: alert.acknowledged_at.or(Some(Utc::now())),
        ..alert
    };

    state.clickhouse_client.insert_alert(&updated_alert).await?;

    info!("Alert {} resolved by user {}", alert_id, user_id);

    Ok(())
}

pub async fn check_alerts(
    state: &AppState,
    tower_id: Uuid,
    data_type: &str,
    value: f64,
) -> Result<Option<alert::AlertMessage>, AppError> {
    let config = &state.config.system;
    let now = Utc::now();

    let (alert_type, threshold, unit) = match data_type {
        "vibration" => (
            alert::AlertType::VibrationAnomaly,
            config.vibration_threshold_mm_s,
            "mm/s",
        ),
        "wind_speed" => (
            alert::AlertType::HighWindSpeed,
            config.wind_speed_threshold_ms,
            "m/s",
        ),
        "ice_thickness" => (
            alert::AlertType::IceDetection,
            config.ice_thickness_threshold_mm,
            "mm",
        ),
        _ => return Ok(None),
    };

    if value < threshold {
        return Ok(None);
    }

    let severity = if value > threshold * 1.5 {
        alert::AlertSeverity::Critical
    } else if value > threshold * 1.2 {
        alert::AlertSeverity::High
    } else if value > threshold * 1.1 {
        alert::AlertSeverity::Medium
    } else {
        alert::AlertSeverity::Low
    };

    let alert = alert::AlertMessage {
        id: Uuid::new_v4(),
        alert_type: alert_type.clone(),
        severity: severity.clone(),
        tower_id: Some(tower_id),
        sensor_id: None,
        title: format!("{:?}告警", alert_type),
        message: format!("{}超过阈值，当前值{:.2}{}，阈值{:.2}{}",
            alert_type, value, unit, threshold, unit),
        timestamp: now,
        data: serde_json::json!({
            "value": value,
            "threshold": threshold,
            "unit": unit,
            "data_type": data_type,
        }),
        acknowledged: false,
        acknowledged_by: None,
        acknowledged_at: None,
    };

    Ok(Some(alert))
}

pub async fn get_alert_summary(
    state: AppState,
) -> Result<alert::AlertSummary, AppError> {
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(24);

    let total = state.clickhouse_client.get_alert_summary(
        None,
        Some(start_time),
        Some(end_time),
    ).await?;

    let critical = state.clickhouse_client.get_alert_summary(
        Some(alert::AlertSeverity::Critical),
        Some(start_time),
        Some(end_time),
    ).await?;

    let high = state.clickhouse_client.get_alert_summary(
        Some(alert::AlertSeverity::High),
        Some(start_time),
        Some(end_time),
    ).await?;

    let active_alerts = state.clickhouse_client.query_alerts(&alert::AlertQuery {
        id: None,
        tower_id: None,
        alert_type: None,
        severity: None,
        status: Some(alert::AlertStatus::Active),
        start_time: Some(start_time),
        end_time: Some(end_time),
        page: Some(0),
        page_size: Some(100),
    }).await?;

    Ok(alert::AlertSummary {
        total_alerts_24h: total,
        critical_alerts: critical,
        high_alerts: high,
        active_alerts: active_alerts.len() as u32,
        acknowledged_alerts: active_alerts.iter().filter(|a| a.acknowledged).count() as u32,
        resolved_alerts_24h: active_alerts.iter().filter(|a| a.resolved).count() as u32,
        avg_response_time_minutes: 0.0,
        top_alert_types: vec![
            (alert::AlertType::IceDetection, total / 3),
            (alert::AlertType::VibrationAnomaly, total / 3),
            (alert::AlertType::HighWindSpeed, total / 3),
        ],
        most_affected_towers: vec![],
        generated_at: end_time,
    })
}

pub async fn check_and_trigger_vibration_alert(
    state: AppState,
    data: &vibration::VibrationReading,
) -> Result<(), AppError> {
    Ok(())
}

pub async fn check_and_trigger_wind_alert(
    state: AppState,
    data: &wind_speed::WindSpeedReading,
) -> Result<(), AppError> {
    Ok(())
}

pub async fn check_and_trigger_ice_alert(
    state: AppState,
    data: &ice_detection::IceDetectionData,
) -> Result<(), AppError> {
    Ok(())
}

pub async fn create_video_review_alert(
    state: &AppState,
    result: &video::VideoVerificationResult,
) -> Result<(), AppError> {
    Ok(())
}

pub async fn confirm_ice_alert(
    state: &AppState,
    result: &video::VideoVerificationResult,
) -> Result<(), AppError> {
    Ok(())
}
