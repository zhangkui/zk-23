use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info};

pub async fn create_sensor(
    state: AppState,
    tower_id: Uuid,
    req: &sensor::CreateSensorRequest,
) -> Result<Uuid, AppError> {
    info!("Creating sensor: {} for tower {}", req.device_id, tower_id);

    let sensor = sensor::Sensor {
        id: Uuid::new_v4(),
        device_id: req.device_id.clone(),
        sensor_type: req.sensor_type.clone(),
        model: req.model.clone(),
        manufacturer: req.manufacturer.clone(),
        status: req.status.clone(),
        calibration_date: req.calibration_date,
        last_calibration_date: req.last_calibration_date,
        calibration_interval_days: req.calibration_interval_days,
        sampling_rate_hz: req.sampling_rate_hz,
        measurement_range_min: req.measurement_range_min,
        measurement_range_max: req.measurement_range_max,
        accuracy: req.accuracy,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state.clickhouse_client.insert_sensor(&sensor).await?;

    Ok(sensor.id)
}

pub async fn get_sensor(
    state: AppState,
    id: Uuid,
) -> Result<sensor::Sensor, AppError> {
    info!("Getting sensor: {}", id);
    Err(AppError::NotFound(format!("Sensor {} not found", id)))
}

pub async fn list_sensors(
    state: AppState,
    tower_id: Option<Uuid>,
    sensor_type: Option<sensor::SensorType>,
    status: Option<sensor::SensorStatus>,
    page: u32,
    page_size: u32,
) -> Result<Vec<sensor::Sensor>, AppError> {
    info!("Listing sensors");

    let towers = state.clickhouse_client.list_towers(&tower::TowerListQuery {
        cable_line_id: None,
        status: None,
        page: Some(0),
        page_size: Some(100),
    }).await?;

    let mut sensors = Vec::new();

    for tower in &towers {
        if let Some(tid) = tower_id {
            if tid != tower.id {
                continue;
            }
        }

        let sensor_types = vec![
            (sensor::SensorType::Vibration, "VIB-001", "振动传感器"),
            (sensor::SensorType::WindSpeed, "WIND-001", "风速传感器"),
            (sensor::SensorType::IceThickness, "ICE-001", "覆冰传感器"),
            (sensor::SensorType::Temperature, "TEMP-001", "温度传感器"),
            (sensor::SensorType::Humidity, "HUM-001", "湿度传感器"),
        ];

        for (stype, prefix, model) in &sensor_types {
            if let Some(req_type) = &sensor_type {
                if req_type != stype {
                    continue;
                }
            }

            sensors.push(sensor::Sensor {
                id: Uuid::new_v4(),
                device_id: format!("{}-{}", prefix, tower.position_in_line),
                sensor_type: *stype,
                model: model.to_string(),
                manufacturer: "国产".to_string(),
                status: status.clone().unwrap_or(sensor::SensorStatus::Active),
                calibration_date: Some(Utc::now() - Duration::days(30)),
                last_calibration_date: Some(Utc::now() - Duration::days(30)),
                calibration_interval_days: 90,
                sampling_rate_hz: match stype {
                    sensor::SensorType::Vibration => 100.0,
                    sensor::SensorType::WindSpeed => 10.0,
                    _ => 1.0,
                },
                measurement_range_min: 0.0,
                measurement_range_max: 100.0,
                accuracy: 0.1,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }
    }

    Ok(sensors.into_iter().skip((page * page_size) as usize).take(page_size as usize).collect())
}

pub async fn update_sensor_status(
    state: AppState,
    id: Uuid,
    status: sensor::SensorStatus,
) -> Result<(), AppError> {
    info!("Updating sensor {} status to {:?}", id, status);
    Ok(())
}

pub async fn calibrate_sensor(
    state: AppState,
    id: Uuid,
    calibrated_by: Uuid,
) -> Result<(), AppError> {
    info!("Calibrating sensor {} by user {}", id, calibrated_by);
    Ok(())
}

pub async fn get_sensor_data(
    state: AppState,
    tower_id: Uuid,
    data_type: &str,
    start_time: chrono::DateTime<Utc>,
    end_time: chrono::DateTime<Utc>,
) -> Result<serde_json::Value, AppError> {
    info!("Getting {} data for tower {} from {:?} to {:?}",
        data_type, tower_id, start_time, end_time);

    match data_type {
        "vibration" => {
            let data = state.clickhouse_client.query_vibration_data(
                &vibration::VibrationQuery {
                    tower_id: Some(tower_id),
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                    frequency_range: None,
                    page: Some(0),
                    page_size: Some(10000),
                },
            ).await?;
            Ok(serde_json::to_value(data)?)
        }
        "wind_speed" => {
            let data = state.clickhouse_client.query_wind_speed_data(
                &vibration::VibrationQuery {
                    tower_id: Some(tower_id),
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                    frequency_range: None,
                    page: Some(0),
                    page_size: Some(10000),
                },
            ).await?;
            Ok(serde_json::to_value(data)?)
        }
        "ice_detection" => {
            let data = state.clickhouse_client.query_ice_detection_data(
                &ice_detection::IceQuery {
                    tower_id: Some(tower_id),
                    start_time: Some(start_time),
                    end_time: Some(end_time),
                    page: Some(0),
                    page_size: Some(10000),
                },
            ).await?;
            Ok(serde_json::to_value(data)?)
        }
        "weather" => {
            let data = state.clickhouse_client.query_weather_data(
                Some(tower_id),
                Some(start_time),
                Some(end_time),
                0,
                10000,
            ).await?;
            Ok(serde_json::to_value(data)?)
        }
        _ => Err(AppError::BadRequest(format!("Unknown data type: {}", data_type))),
    }
}
