use crate::{AppState, AppError, models::*, mq, services, websocket};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, error, warn};
use rand::Rng;

pub async fn start_data_collection(state: AppState) -> Result<(), AppError> {
    info!("Starting data collection service...");

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_sensor_simulation(state_clone).await {
            error!("Sensor simulation error: {}", e);
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_weather_simulation(state_clone).await {
            error!("Weather simulation error: {}", e);
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_periodic_analysis(state_clone).await {
            error!("Periodic analysis error: {}", e);
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_heartbeat(state_clone).await {
            error!("Heartbeat error: {}", e);
        }
    });

    info!("Data collection service started");
    Ok(())
}

async fn start_sensor_simulation(state: AppState) -> Result<(), AppError> {
    info!("Starting sensor data simulation...");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let towers = match state.clickhouse_client.list_towers(&tower::TowerListQuery {
            cable_line_id: None,
            status: Some(tower::TowerStatus::Operational),
            page: Some(0),
            page_size: Some(100),
        }).await {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to fetch towers: {}", e);
                continue;
            }
        };

        for tower in &towers {
            if let Err(e) = generate_and_publish_sensor_data(&state, tower).await {
                error!("Error generating sensor data for tower {}: {}", tower.id, e);
            }
        }
    }
}

async fn generate_and_publish_sensor_data(
    state: &AppState,
    tower: &tower::Tower,
) -> Result<(), AppError> {
    use sensor::ReadingQuality;
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let now = Utc::now();
    let sensor_id = Uuid::new_v4();

    let base_vibration = 1.5 + (now.timestamp() as f64 * 0.0001).sin() * 0.5;
    let vibration_noise: f64 = rng.gen_range(-0.3..0.3);
    let vibration_velocity = (base_vibration + vibration_noise).max(0.0);

    let vibration_reading = vibration::VibrationReading {
        sensor_id,
        tower_id: tower.id,
        timestamp: now,
        frequency_hz: 2.5 + rng.gen_range(-0.2..0.2),
        amplitude_mm_s: vibration_velocity * 0.1,
        velocity_mm_s: vibration_velocity,
        acceleration_mm_s2: vibration_velocity * 10.0,
        displacement_mm: vibration_velocity * 0.05,
        direction: Some(rng.gen_range(0.0..360.0)),
        temperature: Some(5.0 + rng.gen_range(-2.0..2.0)),
        raw_spectrum: Some(vec![0.1, 0.5, 1.2, 0.8, 0.3]),
    };

    mq::publisher::publish_vibration_reading(state, &vibration_reading).await?;

    let base_wind = 10.0 + (now.timestamp() as f64 * 0.00005).sin() * 8.0;
    let wind_noise: f64 = rng.gen_range(-2.0..2.0);
    let wind_speed = (base_wind + wind_noise).max(0.0);

    let wind_reading = wind_speed::WindSpeedReading {
        sensor_id,
        tower_id: tower.id,
        timestamp: now,
        wind_speed_ms: wind_speed,
        wind_direction_deg: rng.gen_range(0.0..360.0),
        gust_speed_ms: Some(wind_speed * 1.5),
        temperature: Some(5.0 + rng.gen_range(-2.0..2.0)),
    };

    mq::publisher::publish_wind_speed_reading(state, &wind_reading).await?;

    let hour = now.hour() as f64;
    let temp_factor = ((hour - 14.0) * 0.5).cos();
    let base_temp = -2.0 + temp_factor * 3.0;
    let temperature = base_temp + rng.gen_range(-1.0..1.0);

    let humidity = 85.0 + rng.gen_range(-10.0..10.0);

    let ice_base = if temperature < 0.0 && humidity > 80.0 {
        (temperature.abs() * 0.5 + (humidity - 80.0) * 0.1).min(15.0)
    } else {
        0.0
    };
    let ice_thickness = (ice_base + rng.gen_range(-1.0..1.0)).max(0.0);

    let ice_reading = ice_detection::IceReading {
        sensor_id,
        tower_id: tower.id,
        timestamp: now,
        ice_thickness_mm: ice_thickness,
        ice_density_kg_m3: if ice_thickness > 0.0 { Some(900.0 + rng.gen_range(-50.0..50.0)) } else { None },
        ambient_temp_c: temperature,
        wind_speed_ms: Some(wind_speed),
        humidity_percent: Some(humidity),
        precipitation_type: if temperature < 0.0 && humidity > 90.0 {
            Some(ice_detection::PrecipitationType::Snow)
        } else if ice_thickness > 0.0 {
            Some(ice_detection::PrecipitationType::FreezingRain)
        } else {
            None
        },
    };

    mq::publisher::publish_ice_detection_reading(state, &ice_reading).await?;

    let weather_data = weather::WeatherData {
        id: Uuid::new_v4(),
        tower_id: tower.id,
        timestamp: now,
        temperature_c: temperature,
        humidity_percent: humidity,
        pressure_hpa: 1013.0 + rng.gen_range(-5.0..5.0),
        wind_speed_ms: wind_speed,
        wind_direction_deg: wind_reading.wind_direction_deg,
        wind_gust_ms: wind_reading.gust_speed_ms,
        precipitation_mm: if ice_thickness > 0.0 { Some(rng.gen_range(0.0..2.0)) } else { Some(0.0) },
        precipitation_type: if temperature < 0.0 { weather::PrecipitationType::Snow } else { weather::PrecipitationType::None },
        visibility_m: Some(10000.0 - ice_thickness * 50.0),
        cloud_cover_percent: Some(80.0 + rng.gen_range(-20.0..20.0)),
        solar_radiation_w_m2: Some(if hour > 6.0 && hour < 18.0 { 200.0 + rng.gen_range(-100.0..100.0) } else { 0.0 }),
        dew_point_c: Some(temperature - 2.0),
        wind_chill_c: Some(temperature - wind_speed * 0.5),
        source: weather::WeatherDataSource::Sensor,
        quality: ReadingQuality::Good,
    };

    state.clickhouse_client.insert_weather_data(&weather_data).await?;

    let risk_level = calculate_risk_level(vibration_velocity, wind_speed, ice_thickness);

    let tower_status = tower::TowerStatusResponse {
        tower_id: tower.id,
        tower_name: tower.name.clone(),
        status: if risk_level == tower::RiskLevel::Critical {
            tower::TowerStatus::Warning
        } else {
            tower::TowerStatus::Operational
        },
        vibration_level: vibration_velocity,
        wind_speed,
        ice_thickness,
        risk_level,
        last_update: now,
    };

    websocket::broadcast_tower_status(state, tower_status).await?;

    Ok(())
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

async fn start_weather_simulation(state: AppState) -> Result<(), AppError> {
    info!("Starting weather data simulation...");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        let forecast = weather::WeatherForecast {
            id: Uuid::new_v4(),
            tower_id: Uuid::nil(),
            forecast_time: Utc::now(),
            forecast_hours: 24,
            hourly_forecast: generate_hourly_forecast(),
            source: "模拟天气预报".to_string(),
            created_at: Utc::now(),
        };

        tracing::debug!("Generated weather forecast for next 24 hours");
    }
}

fn generate_hourly_forecast() -> Vec<weather::HourlyForecast> {
    let mut rng = rand::thread_rng();
    let mut forecast = Vec::new();
    let now = Utc::now();

    for i in 0..24 {
        let hour = (now.hour() as i32 + i as i32) % 24;
        let temp_factor = ((hour as f64 - 14.0) * 0.5).cos();
        let temperature = -2.0 + temp_factor * 3.0 + rng.gen_range(-1.0..1.0);

        forecast.push(weather::HourlyForecast {
            timestamp: now + Duration::hours(i as i64),
            temperature_c: temperature,
            humidity_percent: 85.0 + rng.gen_range(-15.0..15.0),
            wind_speed_ms: 8.0 + rng.gen_range(-3.0..5.0),
            wind_direction_deg: rng.gen_range(0.0..360.0),
            wind_gust_ms: Some(12.0 + rng.gen_range(-2.0..4.0)),
            precipitation_probability_percent: if temperature < 0.0 { 70.0 } else { 30.0 },
            precipitation_mm: if temperature < 0.0 { rng.gen_range(0.0..3.0) } else { rng.gen_range(0.0..1.0) },
            precipitation_type: if temperature < 0.0 {
                weather::PrecipitationType::Snow
            } else {
                weather::PrecipitationType::Rain
            },
            cloud_cover_percent: 70.0 + rng.gen_range(-30.0..30.0),
        });
    }

    forecast
}

async fn start_periodic_analysis(state: AppState) -> Result<(), AppError> {
    info!("Starting periodic analysis service...");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;

        let towers = match state.clickhouse_client.list_towers(&tower::TowerListQuery {
            cable_line_id: None,
            status: None,
            page: Some(0),
            page_size: Some(100),
        }).await {
            Ok(t) => t,
            Err(e) => {
                warn!("Failed to fetch towers for analysis: {}", e);
                continue;
            }
        };

        for tower in &towers {
            let analysis_result = services::ice_risk::analyze_ice_risk(state.clone(), tower.id).await;
            match analysis_result {
                Ok(result) => {
                    tracing::debug!("Ice analysis for tower {}: risk={:?}", tower.id, result.risk_level);
                }
                Err(e) => {
                    warn!("Ice analysis failed for tower {}: {}", tower.id, e);
                }
            }
        }
    }
}

async fn start_heartbeat(state: AppState) -> Result<(), AppError> {
    info!("Starting system heartbeat...");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;
        if let Err(e) = mq::publisher::publish_heartbeat(&state).await {
            error!("Failed to publish heartbeat: {}", e);
        }
    }
}
