use crate::{AppState, AppError, models::*, services, mq};
use futures::StreamExt;
use tracing::{info, error, warn};
use uuid::Uuid;

pub async fn start_subscribers(state: AppState) -> Result<(), AppError> {
    info!("Starting NATS subscribers...");

    let nats_client = state.nats_client.clone();
    let state_clone = state.clone();
    let vibration_subscriber = nats_client.subscribe(mq::nats_client::subjects::VIBRATION_DATA).await?;
    tokio::spawn(async move {
        process_vibration_data(state_clone, vibration_subscriber).await;
    });

    let state_clone = state.clone();
    let wind_subscriber = nats_client.subscribe(mq::nats_client::subjects::WIND_SPEED_DATA).await?;
    tokio::spawn(async move {
        process_wind_speed_data(state_clone, wind_subscriber).await;
    });

    let state_clone = state.clone();
    let ice_subscriber = nats_client.subscribe(mq::nats_client::subjects::ICE_DETECTION_DATA).await?;
    tokio::spawn(async move {
        process_ice_detection_data(state_clone, ice_subscriber).await;
    });

    let state_clone = state.clone();
    let weather_subscriber = nats_client.subscribe(mq::nats_client::subjects::WEATHER_DATA).await?;
    tokio::spawn(async move {
        process_weather_data(state_clone, weather_subscriber).await;
    });

    let state_clone = state.clone();
    let alert_triggered_subscriber = nats_client.subscribe(mq::nats_client::subjects::ALERT_TRIGGERED).await?;
    tokio::spawn(async move {
        process_alert_triggered(state_clone, alert_triggered_subscriber).await;
    });

    let state_clone = state.clone();
    let video_verification_completed_subscriber = nats_client.subscribe(mq::nats_client::subjects::VIDEO_VERIFICATION_COMPLETED).await?;
    tokio::spawn(async move {
        process_video_verification_completed(state_clone, video_verification_completed_subscriber).await;
    });

    info!("All NATS subscribers started successfully");
    Ok(())
}

async fn process_vibration_data(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<vibration::VibrationReading>(&payload) {
            Ok(reading) => {
                if let Err(e) = handle_vibration_reading(&state, reading).await {
                    error!("Error handling vibration reading: {}", e);
                }
            }
            Err(e) => {
                warn!("Invalid vibration data received: {}", e);
            }
        }
    }
}

async fn handle_vibration_reading(
    state: &AppState,
    reading: vibration::VibrationReading,
) -> Result<(), AppError> {
    use crate::models::sensor::ReadingQuality;

    let data = vibration::VibrationData {
        id: Uuid::new_v4(),
        tower_id: reading.tower_id,
        sensor_id: reading.sensor_id,
        timestamp: reading.timestamp,
        frequency_hz: reading.frequency_hz,
        amplitude_mm_s: reading.amplitude_mm_s,
        velocity_mm_s: reading.velocity_mm_s,
        acceleration_mm_s2: reading.acceleration_mm_s2,
        displacement_mm: reading.displacement_mm,
        direction: reading.direction,
        temperature: reading.temperature,
        quality: ReadingQuality::Good,
        raw_spectrum: reading.raw_spectrum,
    };

    state.clickhouse_client.insert_vibration_data(&data).await?;

    if reading.velocity_mm_s > state.config.system.vibration_alert_threshold_mms {
        services::alert::check_and_trigger_vibration_alert(state, &data).await?;
    }

    Ok(())
}

async fn process_wind_speed_data(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<wind_speed::WindSpeedReading>(&payload) {
            Ok(reading) => {
                if let Err(e) = handle_wind_speed_reading(&state, reading).await {
                    error!("Error handling wind speed reading: {}", e);
                }
            }
            Err(e) => {
                warn!("Invalid wind speed data received: {}", e);
            }
        }
    }
}

async fn handle_wind_speed_reading(
    state: &AppState,
    reading: wind_speed::WindSpeedReading,
) -> Result<(), AppError> {
    use crate::models::sensor::ReadingQuality;

    let data = wind_speed::WindSpeedData {
        id: Uuid::new_v4(),
        tower_id: reading.tower_id,
        sensor_id: reading.sensor_id,
        timestamp: reading.timestamp,
        wind_speed_ms: reading.wind_speed_ms,
        wind_direction_deg: reading.wind_direction_deg,
        gust_speed_ms: reading.gust_speed_ms,
        temperature: reading.temperature,
        quality: ReadingQuality::Good,
    };

    state.clickhouse_client.insert_wind_speed_data(&data).await?;

    if reading.wind_speed_ms > state.config.system.wind_speed_threshold_ms {
        services::alert::check_and_trigger_wind_alert(state, &data).await?;
    }

    Ok(())
}

async fn process_ice_detection_data(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<ice_detection::IceReading>(&payload) {
            Ok(reading) => {
                if let Err(e) = handle_ice_detection_reading(&state, reading).await {
                    error!("Error handling ice detection reading: {}", e);
                }
            }
            Err(e) => {
                warn!("Invalid ice detection data received: {}", e);
            }
        }
    }
}

async fn handle_ice_detection_reading(
    state: &AppState,
    reading: ice_detection::IceReading,
) -> Result<(), AppError> {
    use crate::models::sensor::ReadingQuality;

    let ice_weight = reading.ice_density_kg_m3.map(|density| {
        density * reading.ice_thickness_mm / 1000.0 * 1.0
    });

    let data = ice_detection::IceDetectionData {
        id: Uuid::new_v4(),
        tower_id: reading.tower_id,
        sensor_id: reading.sensor_id,
        timestamp: reading.timestamp,
        ice_thickness_mm: reading.ice_thickness_mm,
        ice_density_kg_m3: reading.ice_density_kg_m3,
        ice_weight_kg: ice_weight,
        ambient_temp_c: reading.ambient_temp_c,
        wind_speed_ms: reading.wind_speed_ms,
        humidity_percent: reading.humidity_percent,
        precipitation_type: reading.precipitation_type.unwrap_or_default(),
        quality: ReadingQuality::Good,
    };

    state.clickhouse_client.insert_ice_detection_data(&data).await?;

    if reading.ice_thickness_mm > state.config.system.ice_alert_threshold_mm {
        services::alert::check_and_trigger_ice_alert(state, &data).await?;
    }

    Ok(())
}

async fn process_weather_data(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<weather::WeatherData>(&payload) {
            Ok(data) => {
                if let Err(e) = state.clickhouse_client.insert_weather_data(&data).await {
                    error!("Error inserting weather data: {}", e);
                }
            }
            Err(e) => {
                warn!("Invalid weather data received: {}", e);
            }
        }
    }
}

async fn process_alert_triggered(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<alert::AlertMessage>(&payload) {
            Ok(alert_msg) => {
                info!("Alert triggered: {} - {}", alert_msg.alert_type, alert_msg.title);

                if let Err(e) = state.alert_tx.send(alert_msg.clone()) {
                    error!("Error broadcasting alert: {}", e);
                }

                if alert_msg.severity >= alert::AlertSeverity::High {
                    if let Err(e) = services::video::request_auto_verification(&state, &alert_msg).await {
                        error!("Error requesting video verification: {}", e);
                    }

                    if let Err(e) = services::shutdown_strategy::evaluate_strategies(&state, alert_msg.tower_id).await {
                        error!("Error evaluating shutdown strategies: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Invalid alert message received: {}", e);
            }
        }
    }
}

async fn process_video_verification_completed(
    state: AppState,
    mut subscriber: async_nats::Subscriber,
) {
    while let Some(message) = subscriber.next().await {
        let payload = message.payload.to_vec();
        match serde_json::from_slice::<video::VideoVerificationResult>(&payload) {
            Ok(result) => {
                info!("Video verification completed for tower: {}", result.tower_id);

                if result.human_review_required && !result.human_reviewed {
                    if let Err(e) = services::alert::create_video_review_alert(&state, &result).await {
                        error!("Error creating video review alert: {}", e);
                    }
                }

                if let Some(ice_verification) = &result.ice_verification {
                    if ice_verification.ice_present
                        && ice_verification.estimated_thickness_mm > state.config.system.ice_alert_threshold_mm
                    {
                        if let Err(e) = services::alert::confirm_ice_alert(&state, &result).await {
                            error!("Error confirming ice alert: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Invalid video verification result received: {}", e);
            }
        }
    }
}
