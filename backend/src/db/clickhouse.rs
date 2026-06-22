use crate::models::*;
use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ClickHouseClient {
    client: Client,
    database: String,
}

impl ClickHouseClient {
    pub async fn new(url: String, database: String) -> Result<Self, crate::AppError> {
        let client = Client::default().with_url(url);
        Ok(Self { client, database })
    }

    pub async fn init_database(&self) -> Result<(), crate::AppError> {
        self.client
            .query(format!("CREATE DATABASE IF NOT EXISTS {}", self.database).as_str())
            .execute()
            .await?;
        Ok(())
    }

    pub async fn create_tables(&self) -> Result<(), crate::AppError> {
        let db = &self.database;

        let queries = vec![
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.towers (
                    id UUID,
                    name String,
                    code String,
                    latitude Float64,
                    longitude Float64,
                    elevation_meters Float64,
                    location_description Nullable(String),
                    height_meters Float64,
                    construction_date Nullable(DateTime),
                    status String,
                    cable_line_id UUID,
                    position_in_line UInt32,
                    max_load_kg Float64,
                    last_inspection_date Nullable(DateTime),
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (cable_line_id, position_in_line, id)"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.sensors (
                    id UUID,
                    device_id String,
                    sensor_type String,
                    model String,
                    manufacturer String,
                    status String,
                    calibration_date Nullable(DateTime),
                    last_calibration_date Nullable(DateTime),
                    calibration_interval_days UInt32,
                    sampling_rate_hz Float64,
                    measurement_range_min Float64,
                    measurement_range_max Float64,
                    accuracy Float64,
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (sensor_type, id)"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.vibration_data (
                    id UUID,
                    tower_id UUID,
                    sensor_id UUID,
                    timestamp DateTime,
                    frequency_hz Float64,
                    amplitude_mm_s Float64,
                    velocity_mm_s Float64,
                    acceleration_mm_s2 Float64,
                    displacement_mm Float64,
                    direction Nullable(Float64),
                    temperature Nullable(Float64),
                    quality String
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(timestamp)
                ORDER BY (tower_id, sensor_id, timestamp)
                TTL timestamp + INTERVAL 1 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.wind_speed_data (
                    id UUID,
                    tower_id UUID,
                    sensor_id UUID,
                    timestamp DateTime,
                    wind_speed_ms Float64,
                    wind_direction_deg Float64,
                    gust_speed_ms Nullable(Float64),
                    temperature Nullable(Float64),
                    quality String
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(timestamp)
                ORDER BY (tower_id, sensor_id, timestamp)
                TTL timestamp + INTERVAL 1 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.ice_detection_data (
                    id UUID,
                    tower_id UUID,
                    sensor_id UUID,
                    timestamp DateTime,
                    ice_thickness_mm Float64,
                    ice_density_kg_m3 Nullable(Float64),
                    ice_weight_kg Nullable(Float64),
                    ambient_temp_c Float64,
                    wind_speed_ms Nullable(Float64),
                    humidity_percent Nullable(Float64),
                    precipitation_type String,
                    quality String
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(timestamp)
                ORDER BY (tower_id, sensor_id, timestamp)
                TTL timestamp + INTERVAL 1 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.weather_data (
                    id UUID,
                    tower_id UUID,
                    timestamp DateTime,
                    temperature_c Float64,
                    humidity_percent Float64,
                    pressure_hpa Float64,
                    wind_speed_ms Float64,
                    wind_direction_deg Float64,
                    wind_gust_ms Nullable(Float64),
                    precipitation_mm Nullable(Float64),
                    precipitation_type String,
                    visibility_m Nullable(Float64),
                    cloud_cover_percent Nullable(Float64),
                    solar_radiation_w_m2 Nullable(Float64),
                    dew_point_c Nullable(Float64),
                    wind_chill_c Nullable(Float64),
                    source String,
                    quality String
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(timestamp)
                ORDER BY (tower_id, timestamp)
                TTL timestamp + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.alerts (
                    id UUID,
                    tower_id UUID,
                    alert_type String,
                    severity String,
                    status String,
                    title String,
                    description String,
                    threshold_value Nullable(Float64),
                    actual_value Nullable(Float64),
                    unit Nullable(String),
                    created_at DateTime,
                    updated_at DateTime,
                    acknowledged_at Nullable(DateTime),
                    acknowledged_by Nullable(UUID),
                    resolved_at Nullable(DateTime),
                    resolved_by Nullable(UUID),
                    resolution_notes Nullable(String),
                    video_verification_id Nullable(UUID)
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(created_at)
                ORDER BY (tower_id, severity, created_at)
                TTL created_at + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.shutdown_logs (
                    id UUID,
                    strategy_id UUID,
                    tower_id UUID,
                    shutdown_type String,
                    start_time DateTime,
                    end_time Nullable(DateTime),
                    duration_minutes Nullable(Int64),
                    reason String,
                    executed_by UUID,
                    passengers_affected Nullable(UInt32),
                    economic_impact Nullable(Float64),
                    temperature_c Float64,
                    wind_speed_ms Float64,
                    wind_direction_deg Float64,
                    ice_thickness_mm Nullable(Float64),
                    visibility_m Nullable(Float64),
                    precipitation_type String,
                    created_at DateTime
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(start_time)
                ORDER BY (tower_id, start_time)
                TTL start_time + INTERVAL 10 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.inspection_records (
                    id UUID,
                    tower_id UUID,
                    inspection_type String,
                    status String,
                    inspector_id UUID,
                    inspector_name String,
                    scheduled_date DateTime,
                    start_time Nullable(DateTime),
                    end_time Nullable(DateTime),
                    duration_minutes Nullable(Int64),
                    overall_condition String,
                    follow_up_required UInt8,
                    follow_up_date Nullable(DateTime),
                    notes Nullable(String),
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(scheduled_date)
                ORDER BY (tower_id, inspection_type, scheduled_date)
                TTL scheduled_date + INTERVAL 10 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.video_verification_results (
                    id UUID,
                    request_id UUID,
                    tower_id UUID,
                    camera_id UUID,
                    verified_by Nullable(UUID),
                    verification_method String,
                    started_at DateTime,
                    completed_at Nullable(DateTime),
                    ice_present UInt8,
                    estimated_thickness_mm Float64,
                    thickness_confidence Float64,
                    ice_type String,
                    ai_confidence Nullable(Float64),
                    human_review_required UInt8,
                    human_reviewed UInt8,
                    reviewed_by Nullable(UUID),
                    reviewed_at Nullable(DateTime),
                    review_notes Nullable(String),
                    created_at DateTime
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(started_at)
                ORDER BY (tower_id, started_at)
                TTL started_at + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.ice_analysis_results (
                    id UUID,
                    tower_id UUID,
                    analysis_time DateTime,
                    time_window_start DateTime,
                    time_window_end DateTime,
                    current_thickness_mm Float64,
                    avg_thickness_mm Float64,
                    max_thickness_mm Float64,
                    min_thickness_mm Float64,
                    thickness_increase_rate_mm_h Float64,
                    estimated_ice_weight_kg Float64,
                    estimated_load_increase_percent Float64,
                    ice_type String,
                    risk_level String,
                    critical_temperature_c Float64,
                    melting_potential Float64,
                    video_verification_recommended UInt8
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(analysis_time)
                ORDER BY (tower_id, analysis_time)
                TTL analysis_time + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.vibration_analysis_results (
                    id UUID,
                    tower_id UUID,
                    analysis_time DateTime,
                    time_window_start DateTime,
                    time_window_end DateTime,
                    avg_velocity Float64,
                    max_velocity Float64,
                    min_velocity Float64,
                    std_velocity Float64,
                    rms_velocity Float64,
                    dominant_frequency Float64,
                    baseline_deviation Float64,
                    vibration_level String,
                    anomaly_detected UInt8,
                    anomaly_details Nullable(String)
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(analysis_time)
                ORDER BY (tower_id, analysis_time)
                TTL analysis_time + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.wind_analysis_results (
                    id UUID,
                    tower_id UUID,
                    analysis_time DateTime,
                    time_window_start DateTime,
                    time_window_end DateTime,
                    avg_speed Float64,
                    max_speed Float64,
                    min_speed Float64,
                    std_speed Float64,
                    avg_direction Float64,
                    direction_variance Float64,
                    max_gust Float64,
                    wind_load_factor Float64,
                    risk_assessment String
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(analysis_time)
                ORDER BY (tower_id, analysis_time)
                TTL analysis_time + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.weather_impact_analyses (
                    id UUID,
                    tower_id UUID,
                    analysis_time DateTime,
                    time_window_start DateTime,
                    time_window_end DateTime,
                    avg_temperature_c Float64,
                    min_temperature_c Float64,
                    max_temperature_c Float64,
                    avg_humidity_percent Float64,
                    avg_wind_speed_ms Float64,
                    max_wind_speed_ms Float64,
                    total_precipitation_mm Float64,
                    precipitation_type String,
                    ice_risk_level String,
                    ice_probability_percent Float64,
                    expected_ice_thickness_mm Float64,
                    wind_risk_level String,
                    wind_probability_percent Float64,
                    expected_max_wind_speed_ms Float64,
                    vibration_risk_level String,
                    vibration_probability_percent Float64,
                    overall_impact_rating String,
                    created_at DateTime
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(analysis_time)
                ORDER BY (tower_id, analysis_time)
                TTL analysis_time + INTERVAL 5 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.sensor_mounts (
                    tower_id UUID,
                    sensor_id UUID,
                    mount_position String,
                    mount_height_meters Float64,
                    direction Nullable(Float64),
                    installed_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (tower_id, sensor_id, installed_at)"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.users (
                    id UUID,
                    username String,
                    email String,
                    full_name String,
                    role String,
                    department String,
                    phone Nullable(String),
                    password_hash String,
                    is_active UInt8,
                    last_login Nullable(DateTime),
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (role, username, id)"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.user_activity_logs (
                    id UUID,
                    user_id UUID,
                    action String,
                    resource_type Nullable(String),
                    resource_id Nullable(UUID),
                    ip_address Nullable(String),
                    user_agent Nullable(String),
                    details String,
                    created_at DateTime
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(created_at)
                ORDER BY (user_id, created_at)
                TTL created_at + INTERVAL 2 YEAR"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.cable_lines (
                    id UUID,
                    name String,
                    code String,
                    description String,
                    start_point String,
                    end_point String,
                    total_length_meters Float64,
                    tower_count UInt32,
                    max_speed_ms Float64,
                    capacity_per_cabin UInt32,
                    status String,
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (id)"
            ),
            format!(
                "CREATE TABLE IF NOT EXISTS {db}.cameras (
                    id UUID,
                    tower_id UUID,
                    device_id String,
                    name String,
                    location String,
                    mount_position String,
                    camera_type String,
                    status String,
                    rtsp_url Nullable(String),
                    http_url Nullable(String),
                    resolution String,
                    fps UInt32,
                    has_ai_analysis UInt8,
                    ai_model_version Nullable(String),
                    last_online Nullable(DateTime),
                    created_at DateTime,
                    updated_at DateTime
                ) ENGINE = MergeTree()
                ORDER BY (tower_id, id)"
            ),
        ];

        for query in queries {
            self.client.query(query.as_str()).execute().await?;
        }

        Ok(())
    }

    pub async fn insert_vibration_data(&self, data: &vibration::VibrationData) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct VibrationRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            frequency_hz: f64,
            amplitude_mm_s: f64,
            velocity_mm_s: f64,
            acceleration_mm_s2: f64,
            displacement_mm: f64,
            direction: Option<f64>,
            temperature: Option<f64>,
            quality: String,
        }

        let row = VibrationRow {
            id: data.id,
            tower_id: data.tower_id,
            sensor_id: data.sensor_id,
            timestamp: data.timestamp,
            frequency_hz: data.frequency_hz,
            amplitude_mm_s: data.amplitude_mm_s,
            velocity_mm_s: data.velocity_mm_s,
            acceleration_mm_s2: data.acceleration_mm_s2,
            displacement_mm: data.displacement_mm,
            direction: data.direction,
            temperature: data.temperature,
            quality: format!("{:?}", data.quality).to_lowercase(),
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.vibration_data", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_wind_speed_data(&self, data: &wind_speed::WindSpeedData) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct WindSpeedRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            wind_speed_ms: f64,
            wind_direction_deg: f64,
            gust_speed_ms: Option<f64>,
            temperature: Option<f64>,
            quality: String,
        }

        let row = WindSpeedRow {
            id: data.id,
            tower_id: data.tower_id,
            sensor_id: data.sensor_id,
            timestamp: data.timestamp,
            wind_speed_ms: data.wind_speed_ms,
            wind_direction_deg: data.wind_direction_deg,
            gust_speed_ms: data.gust_speed_ms,
            temperature: data.temperature,
            quality: format!("{:?}", data.quality).to_lowercase(),
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.wind_speed_data", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_ice_detection_data(&self, data: &ice_detection::IceDetectionData) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct IceDetectionRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            ice_thickness_mm: f64,
            ice_density_kg_m3: Option<f64>,
            ice_weight_kg: Option<f64>,
            ambient_temp_c: f64,
            wind_speed_ms: Option<f64>,
            humidity_percent: Option<f64>,
            precipitation_type: String,
            quality: String,
        }

        let row = IceDetectionRow {
            id: data.id,
            tower_id: data.tower_id,
            sensor_id: data.sensor_id,
            timestamp: data.timestamp,
            ice_thickness_mm: data.ice_thickness_mm,
            ice_density_kg_m3: data.ice_density_kg_m3,
            ice_weight_kg: data.ice_weight_kg,
            ambient_temp_c: data.ambient_temp_c,
            wind_speed_ms: data.wind_speed_ms,
            humidity_percent: data.humidity_percent,
            precipitation_type: format!("{:?}", data.precipitation_type).to_lowercase(),
            quality: format!("{:?}", data.quality).to_lowercase(),
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.ice_detection_data", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_weather_data(&self, data: &weather::WeatherData) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct WeatherRow {
            id: Uuid,
            tower_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            temperature_c: f64,
            humidity_percent: f64,
            pressure_hpa: f64,
            wind_speed_ms: f64,
            wind_direction_deg: f64,
            wind_gust_ms: Option<f64>,
            precipitation_mm: Option<f64>,
            precipitation_type: String,
            visibility_m: Option<f64>,
            cloud_cover_percent: Option<f64>,
            solar_radiation_w_m2: Option<f64>,
            dew_point_c: Option<f64>,
            wind_chill_c: Option<f64>,
            source: String,
            quality: String,
        }

        let row = WeatherRow {
            id: data.id,
            tower_id: data.tower_id,
            timestamp: data.timestamp,
            temperature_c: data.temperature_c,
            humidity_percent: data.humidity_percent,
            pressure_hpa: data.pressure_hpa,
            wind_speed_ms: data.wind_speed_ms,
            wind_direction_deg: data.wind_direction_deg,
            wind_gust_ms: data.wind_gust_ms,
            precipitation_mm: data.precipitation_mm,
            precipitation_type: format!("{:?}", data.precipitation_type).to_lowercase(),
            visibility_m: data.visibility_m,
            cloud_cover_percent: data.cloud_cover_percent,
            solar_radiation_w_m2: data.solar_radiation_w_m2,
            dew_point_c: data.dew_point_c,
            wind_chill_c: data.wind_chill_c,
            source: format!("{:?}", data.source).to_lowercase(),
            quality: format!("{:?}", data.quality).to_lowercase(),
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.weather_data", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_alert(&self, alert: &alert::Alert) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct AlertRow {
            id: Uuid,
            tower_id: Uuid,
            alert_type: String,
            severity: String,
            status: String,
            title: String,
            description: String,
            threshold_value: Option<f64>,
            actual_value: Option<f64>,
            unit: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
            acknowledged_by: Option<Uuid>,
            resolved_at: Option<chrono::DateTime<chrono::Utc>>,
            resolved_by: Option<Uuid>,
            resolution_notes: Option<String>,
            video_verification_id: Option<Uuid>,
        }

        let row = AlertRow {
            id: alert.id,
            tower_id: alert.tower_id,
            alert_type: format!("{:?}", alert.alert_type).to_lowercase(),
            severity: format!("{:?}", alert.severity).to_lowercase(),
            status: format!("{:?}", alert.status).to_lowercase(),
            title: alert.title.clone(),
            description: alert.description.clone(),
            threshold_value: alert.threshold_value,
            actual_value: alert.actual_value,
            unit: alert.unit.clone(),
            created_at: alert.created_at,
            updated_at: alert.updated_at,
            acknowledged_at: alert.acknowledged_at,
            acknowledged_by: alert.acknowledged_by,
            resolved_at: alert.resolved_at,
            resolved_by: alert.resolved_by,
            resolution_notes: alert.resolution_notes.clone(),
            video_verification_id: alert.video_verification_id,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.alerts", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn query_vibration_data(
        &self,
        tower_id: Uuid,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<vibration::VibrationData>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct VibrationRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            frequency_hz: f64,
            amplitude_mm_s: f64,
            velocity_mm_s: f64,
            acceleration_mm_s2: f64,
            displacement_mm: f64,
            direction: Option<f64>,
            temperature: Option<f64>,
            quality: String,
        }

        let query = format!(
            "SELECT id, tower_id, sensor_id, timestamp, frequency_hz, amplitude_mm_s,
             velocity_mm_s, acceleration_mm_s2, displacement_mm, direction, temperature, quality
             FROM {}.vibration_data
             WHERE tower_id = ? AND timestamp >= ? AND timestamp <= ?
             ORDER BY timestamp ASC",
            self.database
        );

        let rows: Vec<VibrationRow> = self
            .client
            .query(query.as_str())
            .bind(tower_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all()
            .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(vibration::VibrationData {
                id: row.id,
                tower_id: row.tower_id,
                sensor_id: row.sensor_id,
                timestamp: row.timestamp,
                frequency_hz: row.frequency_hz,
                amplitude_mm_s: row.amplitude_mm_s,
                velocity_mm_s: row.velocity_mm_s,
                acceleration_mm_s2: row.acceleration_mm_s2,
                displacement_mm: row.displacement_mm,
                direction: row.direction,
                temperature: row.temperature,
                quality: serde_json::from_str::<sensor::ReadingQuality>(&format!("\"{}\"", row.quality)).unwrap_or(sensor::ReadingQuality::Good),
                raw_spectrum: None,
            });
        }

        Ok(result)
    }

    pub async fn query_wind_speed_data(
        &self,
        tower_id: Uuid,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<wind_speed::WindSpeedData>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct WindSpeedRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            wind_speed_ms: f64,
            wind_direction_deg: f64,
            gust_speed_ms: Option<f64>,
            temperature: Option<f64>,
            quality: String,
        }

        let query = format!(
            "SELECT id, tower_id, sensor_id, timestamp, wind_speed_ms, wind_direction_deg,
             gust_speed_ms, temperature, quality
             FROM {}.wind_speed_data
             WHERE tower_id = ? AND timestamp >= ? AND timestamp <= ?
             ORDER BY timestamp ASC",
            self.database
        );

        let rows: Vec<WindSpeedRow> = self
            .client
            .query(query.as_str())
            .bind(tower_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all()
            .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(wind_speed::WindSpeedData {
                id: row.id,
                tower_id: row.tower_id,
                sensor_id: row.sensor_id,
                timestamp: row.timestamp,
                wind_speed_ms: row.wind_speed_ms,
                wind_direction_deg: row.wind_direction_deg,
                gust_speed_ms: row.gust_speed_ms,
                temperature: row.temperature,
                quality: serde_json::from_str::<sensor::ReadingQuality>(&format!("\"{}\"", row.quality)).unwrap_or(sensor::ReadingQuality::Good),
            });
        }

        Ok(result)
    }

    pub async fn query_ice_detection_data(
        &self,
        tower_id: Uuid,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ice_detection::IceDetectionData>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct IceDetectionRow {
            id: Uuid,
            tower_id: Uuid,
            sensor_id: Uuid,
            timestamp: chrono::DateTime<chrono::Utc>,
            ice_thickness_mm: f64,
            ice_density_kg_m3: Option<f64>,
            ice_weight_kg: Option<f64>,
            ambient_temp_c: f64,
            wind_speed_ms: Option<f64>,
            humidity_percent: Option<f64>,
            precipitation_type: String,
            quality: String,
        }

        let query = format!(
            "SELECT id, tower_id, sensor_id, timestamp, ice_thickness_mm, ice_density_kg_m3,
             ice_weight_kg, ambient_temp_c, wind_speed_ms, humidity_percent, precipitation_type, quality
             FROM {}.ice_detection_data
             WHERE tower_id = ? AND timestamp >= ? AND timestamp <= ?
             ORDER BY timestamp ASC",
            self.database
        );

        let rows: Vec<IceDetectionRow> = self
            .client
            .query(query.as_str())
            .bind(tower_id)
            .bind(start_time)
            .bind(end_time)
            .fetch_all()
            .await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(ice_detection::IceDetectionData {
                id: row.id,
                tower_id: row.tower_id,
                sensor_id: row.sensor_id,
                timestamp: row.timestamp,
                ice_thickness_mm: row.ice_thickness_mm,
                ice_density_kg_m3: row.ice_density_kg_m3,
                ice_weight_kg: row.ice_weight_kg,
                ambient_temp_c: row.ambient_temp_c,
                wind_speed_ms: row.wind_speed_ms,
                humidity_percent: row.humidity_percent,
                precipitation_type: serde_json::from_str::<ice_detection::PrecipitationType>(&format!("\"{}\"", row.precipitation_type)).unwrap_or_default(),
                quality: serde_json::from_str::<sensor::ReadingQuality>(&format!("\"{}\"", row.quality)).unwrap_or(sensor::ReadingQuality::Good),
            });
        }

        Ok(result)
    }

    pub async fn query_alerts(
        &self,
        query: &alert::AlertQuery,
    ) -> Result<Vec<alert::Alert>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct AlertRow {
            id: Uuid,
            tower_id: Uuid,
            alert_type: String,
            severity: String,
            status: String,
            title: String,
            description: String,
            threshold_value: Option<f64>,
            actual_value: Option<f64>,
            unit: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
            acknowledged_at: Option<chrono::DateTime<chrono::Utc>>,
            acknowledged_by: Option<Uuid>,
            resolved_at: Option<chrono::DateTime<chrono::Utc>>,
            resolved_by: Option<Uuid>,
            resolution_notes: Option<String>,
            video_verification_id: Option<Uuid>,
        }

        let mut sql = format!(
            "SELECT id, tower_id, alert_type, severity, status, title, description,
             threshold_value, actual_value, unit, created_at, updated_at,
             acknowledged_at, acknowledged_by, resolved_at, resolved_by,
             resolution_notes, video_verification_id
             FROM {}.alerts
             WHERE 1=1",
            self.database
        );

        let mut params: Vec<String> = Vec::new();

        if let Some(tower_id) = query.tower_id {
            sql.push_str(" AND tower_id = ?");
            params.push(tower_id.to_string());
        }
        if let Some(status) = query.status {
            sql.push_str(" AND status = ?");
            params.push(format!("{:?}", status).to_lowercase());
        }
        if let Some(severity) = query.severity {
            sql.push_str(" AND severity = ?");
            params.push(format!("{:?}", severity).to_lowercase());
        }
        if let Some(start_time) = query.start_time {
            sql.push_str(" AND created_at >= ?");
            params.push(start_time.to_rfc3339());
        }
        if let Some(end_time) = query.end_time {
            sql.push_str(" AND created_at <= ?");
            params.push(end_time.to_rfc3339());
        }

        sql.push_str(" ORDER BY created_at DESC");

        let page = query.page.unwrap_or(0);
        let page_size = query.page_size.unwrap_or(50);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, page * page_size));

        let mut query_builder = self.client.query(sql.as_str());
        for param in &params {
            query_builder = query_builder.bind(param);
        }

        let rows: Vec<AlertRow> = query_builder.fetch_all().await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(alert::Alert {
                id: row.id,
                tower_id: row.tower_id,
                alert_type: serde_json::from_str::<alert::AlertType>(&format!("\"{}\"", row.alert_type)).unwrap_or(alert::AlertType::SystemError),
                severity: serde_json::from_str::<alert::AlertSeverity>(&format!("\"{}\"", row.severity)).unwrap_or_default(),
                status: serde_json::from_str::<alert::AlertStatus>(&format!("\"{}\"", row.status)).unwrap_or_default(),
                title: row.title,
                description: row.description,
                triggered_by: alert::AlertTrigger::System { component: "system".to_string(), error_code: "".to_string() },
                threshold_value: row.threshold_value,
                actual_value: row.actual_value,
                unit: row.unit,
                created_at: row.created_at,
                updated_at: row.updated_at,
                acknowledged_at: row.acknowledged_at,
                acknowledged_by: row.acknowledged_by,
                resolved_at: row.resolved_at,
                resolved_by: row.resolved_by,
                resolution_notes: row.resolution_notes,
                video_verification_id: row.video_verification_id,
                related_data_ids: Vec::new(),
            });
        }

        Ok(result)
    }

    pub async fn get_alert_summary(&self) -> Result<alert::AlertSummary, crate::AppError> {
        let db = &self.database;

        let total_open: u32 = self
            .client
            .query(format!("SELECT count() FROM {}.alerts WHERE status = 'open'", db).as_str())
            .fetch_one()
            .await?;

        let total_acknowledged: u32 = self
            .client
            .query(format!("SELECT count() FROM {}.alerts WHERE status = 'acknowledged'", db).as_str())
            .fetch_one()
            .await?;

        let last_24h_count: u32 = self
            .client
            .query(format!("SELECT count() FROM {}.alerts WHERE created_at >= now() - INTERVAL 24 HOUR", db).as_str())
            .fetch_one()
            .await?;

        let unresolved_critical: u32 = self
            .client
            .query(format!("SELECT count() FROM {}.alerts WHERE severity IN ('critical', 'emergency') AND status IN ('open', 'acknowledged', 'in_progress')", db).as_str())
            .fetch_one()
            .await?;

        Ok(alert::AlertSummary {
            total_open,
            total_acknowledged,
            by_severity: std::collections::HashMap::new(),
            by_type: std::collections::HashMap::new(),
            last_24h_count,
            unresolved_critical,
        })
    }

    pub async fn insert_inspection(&self, inspection: &inspection::InspectionRecord) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct InspectionRow {
            id: Uuid,
            tower_id: Uuid,
            inspection_type: String,
            status: String,
            inspector_id: Uuid,
            inspector_name: String,
            scheduled_date: chrono::DateTime<chrono::Utc>,
            start_time: Option<chrono::DateTime<chrono::Utc>>,
            end_time: Option<chrono::DateTime<chrono::Utc>>,
            duration_minutes: Option<i64>,
            overall_condition: String,
            follow_up_required: u8,
            follow_up_date: Option<chrono::DateTime<chrono::Utc>>,
            notes: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = InspectionRow {
            id: inspection.id,
            tower_id: inspection.tower_id,
            inspection_type: format!("{:?}", inspection.inspection_type).to_lowercase(),
            status: format!("{:?}", inspection.status).to_lowercase(),
            inspector_id: inspection.inspector_id,
            inspector_name: inspection.inspector_name.clone(),
            scheduled_date: inspection.scheduled_date,
            start_time: inspection.start_time,
            end_time: inspection.end_time,
            duration_minutes: inspection.duration_minutes,
            overall_condition: format!("{:?}", inspection.overall_condition).to_lowercase(),
            follow_up_required: if inspection.follow_up_required { 1 } else { 0 },
            follow_up_date: inspection.follow_up_date,
            notes: inspection.notes.clone(),
            created_at: inspection.created_at,
            updated_at: inspection.updated_at,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.inspection_records", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_shutdown_log(&self, log: &shutdown_strategy::ShutdownLog) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct ShutdownLogRow {
            id: Uuid,
            strategy_id: Uuid,
            tower_id: Uuid,
            shutdown_type: String,
            start_time: chrono::DateTime<chrono::Utc>,
            end_time: Option<chrono::DateTime<chrono::Utc>>,
            duration_minutes: Option<i64>,
            reason: String,
            executed_by: Uuid,
            passengers_affected: Option<u32>,
            economic_impact: Option<f64>,
            temperature_c: f64,
            wind_speed_ms: f64,
            wind_direction_deg: f64,
            ice_thickness_mm: Option<f64>,
            visibility_m: Option<f64>,
            precipitation_type: String,
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let row = ShutdownLogRow {
            id: log.id,
            strategy_id: log.strategy_id,
            tower_id: log.tower_id,
            shutdown_type: format!("{:?}", log.shutdown_type).to_lowercase(),
            start_time: log.start_time,
            end_time: log.end_time,
            duration_minutes: log.duration_minutes,
            reason: log.reason.clone(),
            executed_by: log.executed_by,
            passengers_affected: log.passengers_affected,
            economic_impact: log.economic_impact,
            temperature_c: log.weather_conditions.temperature_c,
            wind_speed_ms: log.weather_conditions.wind_speed_ms,
            wind_direction_deg: log.weather_conditions.wind_direction_deg,
            ice_thickness_mm: log.weather_conditions.ice_thickness_mm,
            visibility_m: log.weather_conditions.visibility_m,
            precipitation_type: format!("{:?}", log.weather_conditions.precipitation_type).to_lowercase(),
            created_at: log.created_at,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.shutdown_logs", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn get_tower(
        &self,
        tower_id: Uuid,
    ) -> Result<Option<tower::Tower>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct TowerRow {
            id: Uuid,
            name: String,
            code: String,
            latitude: f64,
            longitude: f64,
            elevation_meters: f64,
            location_description: Option<String>,
            height_meters: f64,
            construction_date: Option<chrono::DateTime<chrono::Utc>>,
            status: String,
            cable_line_id: Uuid,
            position_in_line: u32,
            max_load_kg: f64,
            last_inspection_date: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let query = format!(
            "SELECT id, name, code, latitude, longitude, elevation_meters, location_description,
             height_meters, construction_date, status, cable_line_id, position_in_line,
             max_load_kg, last_inspection_date, created_at, updated_at
             FROM {}.towers
             WHERE id = ?",
            self.database
        );

        let row: Option<TowerRow> = self
            .client
            .query(query.as_str())
            .bind(tower_id)
            .fetch_optional()
            .await?;

        Ok(row.map(|r| tower::Tower {
            id: r.id,
            name: r.name,
            code: r.code,
            location: tower::Location {
                latitude: r.latitude,
                longitude: r.longitude,
                elevation_meters: r.elevation_meters,
                description: r.location_description,
            },
            height_meters: r.height_meters,
            construction_date: r.construction_date,
            status: serde_json::from_str::<tower::TowerStatus>(&format!("\"{}\"", r.status)).unwrap_or(tower::TowerStatus::Operational),
            cable_line_id: r.cable_line_id,
            position_in_line: r.position_in_line,
            max_load_kg: r.max_load_kg,
            last_inspection_date: r.last_inspection_date,
            created_at: r.created_at,
            updated_at: r.updated_at,
            sensors: Vec::new(),
        }))
    }

    pub async fn list_towers(
        &self,
        query: &tower::TowerListQuery,
    ) -> Result<Vec<tower::Tower>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct TowerRow {
            id: Uuid,
            name: String,
            code: String,
            latitude: f64,
            longitude: f64,
            elevation_meters: f64,
            location_description: Option<String>,
            height_meters: f64,
            construction_date: Option<chrono::DateTime<chrono::Utc>>,
            status: String,
            cable_line_id: Uuid,
            position_in_line: u32,
            max_load_kg: f64,
            last_inspection_date: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let mut sql = format!(
            "SELECT id, name, code, latitude, longitude, elevation_meters, location_description,
             height_meters, construction_date, status, cable_line_id, position_in_line,
             max_load_kg, last_inspection_date, created_at, updated_at
             FROM {}.towers
             WHERE 1=1",
            self.database
        );

        if let Some(cable_line_id) = query.cable_line_id {
            sql.push_str(&format!(" AND cable_line_id = '{}'", cable_line_id));
        }
        if let Some(status) = query.status {
            sql.push_str(&format!(" AND status = '{:?}'", status).to_lowercase());
        }

        sql.push_str(" ORDER BY position_in_line ASC");

        let page = query.page.unwrap_or(0);
        let page_size = query.page_size.unwrap_or(100);
        sql.push_str(&format!(" LIMIT {} OFFSET {}", page_size, page * page_size));

        let rows: Vec<TowerRow> = self.client.query(sql.as_str()).fetch_all().await?;

        let mut result = Vec::new();
        for row in rows {
            result.push(tower::Tower {
                id: row.id,
                name: row.name,
                code: row.code,
                location: tower::Location {
                    latitude: row.latitude,
                    longitude: row.longitude,
                    elevation_meters: row.elevation_meters,
                    description: row.location_description,
                },
                height_meters: row.height_meters,
                construction_date: row.construction_date,
                status: serde_json::from_str::<tower::TowerStatus>(&format!("\"{}\"", row.status)).unwrap_or(tower::TowerStatus::Operational),
                cable_line_id: row.cable_line_id,
                position_in_line: row.position_in_line,
                max_load_kg: row.max_load_kg,
                last_inspection_date: row.last_inspection_date,
                created_at: row.created_at,
                updated_at: row.updated_at,
                sensors: Vec::new(),
            });
        }

        Ok(result)
    }

    pub async fn insert_tower(&self, tower: &tower::Tower) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct TowerRow {
            id: Uuid,
            name: String,
            code: String,
            latitude: f64,
            longitude: f64,
            elevation_meters: f64,
            location_description: Option<String>,
            height_meters: f64,
            construction_date: Option<chrono::DateTime<chrono::Utc>>,
            status: String,
            cable_line_id: Uuid,
            position_in_line: u32,
            max_load_kg: f64,
            last_inspection_date: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = TowerRow {
            id: tower.id,
            name: tower.name.clone(),
            code: tower.code.clone(),
            latitude: tower.location.latitude,
            longitude: tower.location.longitude,
            elevation_meters: tower.location.elevation_meters,
            location_description: tower.location.description.clone(),
            height_meters: tower.height_meters,
            construction_date: tower.construction_date,
            status: format!("{:?}", tower.status).to_lowercase(),
            cable_line_id: tower.cable_line_id,
            position_in_line: tower.position_in_line,
            max_load_kg: tower.max_load_kg,
            last_inspection_date: tower.last_inspection_date,
            created_at: tower.created_at,
            updated_at: tower.updated_at,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.towers", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_sensor(&self, sensor: &sensor::Sensor) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct SensorRow {
            id: Uuid,
            device_id: String,
            sensor_type: String,
            model: String,
            manufacturer: String,
            status: String,
            calibration_date: Option<chrono::DateTime<chrono::Utc>>,
            last_calibration_date: Option<chrono::DateTime<chrono::Utc>>,
            calibration_interval_days: u32,
            sampling_rate_hz: f64,
            measurement_range_min: f64,
            measurement_range_max: f64,
            accuracy: f64,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = SensorRow {
            id: sensor.id,
            device_id: sensor.device_id.clone(),
            sensor_type: format!("{:?}", sensor.sensor_type).to_lowercase(),
            model: sensor.model.clone(),
            manufacturer: sensor.manufacturer.clone(),
            status: format!("{:?}", sensor.status).to_lowercase(),
            calibration_date: sensor.calibration_date,
            last_calibration_date: sensor.last_calibration_date,
            calibration_interval_days: sensor.calibration_interval_days,
            sampling_rate_hz: sensor.sampling_rate_hz,
            measurement_range_min: sensor.measurement_range_min,
            measurement_range_max: sensor.measurement_range_max,
            accuracy: sensor.accuracy,
            created_at: sensor.created_at,
            updated_at: sensor.updated_at,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.sensors", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn insert_user(&self, user: &user::User, password_hash: &str) -> Result<(), crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct UserRow {
            id: Uuid,
            username: String,
            email: String,
            full_name: String,
            role: String,
            department: String,
            phone: Option<String>,
            password_hash: String,
            is_active: u8,
            last_login: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let row = UserRow {
            id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            full_name: user.full_name.clone(),
            role: format!("{:?}", user.role).to_lowercase(),
            department: user.department.clone(),
            phone: user.phone.clone(),
            password_hash: password_hash.to_string(),
            is_active: if user.is_active { 1 } else { 0 },
            last_login: user.last_login,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        let mut inserter = self
            .client
            .inserter(format!("{}.users", self.database).as_str())?
            .with_timeouts(Some(std::time::Duration::from_secs(5)), None);

        inserter.write(&row).await?;
        inserter.end().await?;
        Ok(())
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<(user::User, String)>, crate::AppError> {
        #[derive(Row, Serialize, Deserialize)]
        struct UserRow {
            id: Uuid,
            username: String,
            email: String,
            full_name: String,
            role: String,
            department: String,
            phone: Option<String>,
            password_hash: String,
            is_active: u8,
            last_login: Option<chrono::DateTime<chrono::Utc>>,
            created_at: chrono::DateTime<chrono::Utc>,
            updated_at: chrono::DateTime<chrono::Utc>,
        }

        let query = format!(
            "SELECT id, username, email, full_name, role, department, phone, password_hash,
             is_active, last_login, created_at, updated_at
             FROM {}.users
             WHERE username = ?",
            self.database
        );

        let row: Option<UserRow> = self
            .client
            .query(query.as_str())
            .bind(username)
            .fetch_optional()
            .await?;

        Ok(row.map(|r| {
            (
                user::User {
                    id: r.id,
                    username: r.username,
                    email: r.email,
                    full_name: r.full_name,
                    role: serde_json::from_str::<user::UserRole>(&format!("\"{}\"", r.role)).unwrap_or(user::UserRole::Viewer),
                    department: r.department,
                    phone: r.phone,
                    is_active: r.is_active == 1,
                    last_login: r.last_login,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                },
                r.password_hash,
            )
        }))
    }
}
