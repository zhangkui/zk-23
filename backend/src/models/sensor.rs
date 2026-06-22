use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sensor {
    pub id: Uuid,
    pub device_id: String,
    pub sensor_type: SensorType,
    pub model: String,
    pub manufacturer: String,
    pub status: SensorStatus,
    pub calibration_date: Option<DateTime<Utc>>,
    pub last_calibration_date: Option<DateTime<Utc>>,
    pub calibration_interval_days: u32,
    pub sampling_rate_hz: f64,
    pub measurement_range_min: f64,
    pub measurement_range_max: f64,
    pub accuracy: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensorType {
    Vibration,
    WindSpeed,
    WindDirection,
    Temperature,
    Humidity,
    IceThickness,
    Tilt,
    Stress,
    Accelerometer,
    Gyroscope,
    GPS,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SensorStatus {
    Active,
    Inactive,
    Calibrating,
    Faulty,
    Maintenance,
    Decommissioned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSensorRequest {
    pub device_id: String,
    pub sensor_type: SensorType,
    pub model: String,
    pub manufacturer: String,
    pub sampling_rate_hz: f64,
    pub measurement_range_min: f64,
    pub measurement_range_max: f64,
    pub accuracy: f64,
    pub calibration_interval_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: Uuid,
    pub tower_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub unit: String,
    pub quality: ReadingQuality,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadingQuality {
    Good,
    Suspect,
    Bad,
    Missing,
}
