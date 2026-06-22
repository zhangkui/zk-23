use crate::{AppState, AppError, models::*, mq};
use chrono::Utc;
use tracing::info;

pub async fn publish_vibration_reading(
    state: &AppState,
    reading: &vibration::VibrationReading,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(reading)?;
    state.nats_client.publish(
        mq::nats_client::subjects::VIBRATION_DATA,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_wind_speed_reading(
    state: &AppState,
    reading: &wind_speed::WindSpeedReading,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(reading)?;
    state.nats_client.publish(
        mq::nats_client::subjects::WIND_SPEED_DATA,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_ice_detection_reading(
    state: &AppState,
    reading: &ice_detection::IceReading,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(reading)?;
    state.nats_client.publish(
        mq::nats_client::subjects::ICE_DETECTION_DATA,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_weather_data(
    state: &AppState,
    data: &weather::WeatherData,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(data)?;
    state.nats_client.publish(
        mq::nats_client::subjects::WEATHER_DATA,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_alert_triggered(
    state: &AppState,
    alert: &alert::AlertMessage,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(alert)?;
    state.nats_client.publish(
        mq::nats_client::subjects::ALERT_TRIGGERED,
        payload,
    ).await?;
    info!("Published alert triggered: {}", alert.title);
    Ok(())
}

pub async fn publish_shutdown_executed(
    state: &AppState,
    shutdown_log: &shutdown_strategy::ShutdownLog,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(shutdown_log)?;
    state.nats_client.publish(
        mq::nats_client::subjects::SHUTDOWN_EXECUTED,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_video_verification_requested(
    state: &AppState,
    request: &video::VideoVerificationRequest,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(request)?;
    state.nats_client.publish(
        mq::nats_client::subjects::VIDEO_VERIFICATION_REQUESTED,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_video_verification_completed(
    state: &AppState,
    result: &video::VideoVerificationResult,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(result)?;
    state.nats_client.publish(
        mq::nats_client::subjects::VIDEO_VERIFICATION_COMPLETED,
        payload,
    ).await?;
    Ok(())
}

pub async fn publish_tower_status_updated(
    state: &AppState,
    tower_status: &tower::TowerStatusResponse,
) -> Result<(), AppError> {
    let payload = serde_json::to_vec(tower_status)?;
    let subject = mq::nats_client::subjects::tower_specific(
        mq::nats_client::subjects::TOWER_STATUS_UPDATED,
        &tower_status.tower_id,
    );
    state.nats_client.publish(&subject, payload).await?;
    Ok(())
}

pub async fn publish_heartbeat(state: &AppState) -> Result<(), AppError> {
    let heartbeat = serde_json::json!({
        "node_id": state.config.node.node_id,
        "node_location": state.config.node.node_location,
        "timestamp": Utc::now().to_rfc3339(),
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    });
    let payload = serde_json::to_vec(&heartbeat)?;
    state.nats_client.publish(
        mq::nats_client::subjects::SYSTEM_HEARTBEAT,
        payload,
    ).await?;
    Ok(())
}
