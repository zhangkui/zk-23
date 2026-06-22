use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationData {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub sensor_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub frequency_hz: f64,
    pub amplitude_mm_s: f64,
    pub velocity_mm_s: f64,
    pub acceleration_mm_s2: f64,
    pub displacement_mm: f64,
    pub direction: Option<f64>,
    pub temperature: Option<f64>,
    pub quality: super::sensor::ReadingQuality,
    pub raw_spectrum: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationReading {
    pub sensor_id: Uuid,
    pub tower_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub frequency_hz: f64,
    pub amplitude_mm_s: f64,
    pub velocity_mm_s: f64,
    pub acceleration_mm_s2: f64,
    pub displacement_mm: f64,
    pub direction: Option<f64>,
    pub temperature: Option<f64>,
    pub raw_spectrum: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationAnalysisResult {
    pub tower_id: Uuid,
    pub analysis_time: DateTime<Utc>,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub avg_velocity: f64,
    pub max_velocity: f64,
    pub min_velocity: f64,
    pub std_velocity: f64,
    pub rms_velocity: f64,
    pub dominant_frequency: f64,
    pub peak_frequencies: Vec<FrequencyPeak>,
    pub baseline_deviation: f64,
    pub vibration_level: VibrationLevel,
    pub anomaly_detected: bool,
    pub anomaly_details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyPeak {
    pub frequency_hz: f64,
    pub amplitude: f64,
    pub harmonic_order: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VibrationLevel {
    Normal,
    Slight,
    Moderate,
    Severe,
    Extreme,
}

impl Default for VibrationLevel {
    fn default() -> Self {
        VibrationLevel::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationQuery {
    pub tower_id: Option<Uuid>,
    pub sensor_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_velocity: Option<f64>,
    pub max_velocity: Option<f64>,
}
