use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceDetectionData {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub sensor_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ice_thickness_mm: f64,
    pub ice_density_kg_m3: Option<f64>,
    pub ice_weight_kg: Option<f64>,
    pub ambient_temp_c: f64,
    pub wind_speed_ms: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_type: Option<PrecipitationType>,
    pub quality: super::sensor::ReadingQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceReading {
    pub sensor_id: Uuid,
    pub tower_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ice_thickness_mm: f64,
    pub ice_density_kg_m3: Option<f64>,
    pub ambient_temp_c: f64,
    pub wind_speed_ms: Option<f64>,
    pub humidity_percent: Option<f64>,
    pub precipitation_type: Option<PrecipitationType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
    Sleet,
    FreezingRain,
    Hail,
}

impl Default for PrecipitationType {
    fn default() -> Self {
        PrecipitationType::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceAnalysisResult {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub analysis_time: DateTime<Utc>,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub avg_ice_thickness_mm: f64,
    pub max_ice_thickness_mm: f64,
    pub ice_type: Option<IceType>,
    pub ice_density_kg_m3: Option<f64>,
    pub temperature_c: f64,
    pub wind_speed_ms: f64,
    pub humidity_percent: f64,
    pub vibration_level_mm_s: f64,
    pub vibration_max_mm_s: f64,
    pub load_increase_percent: f64,
    pub growth_rate_mm_h: f64,
    pub risk_level: IceRiskLevel,
    pub risk_score: f64,
    pub contributing_factors: Vec<RiskFactor>,
    pub predictions: Vec<IcePrediction>,
    pub mitigation_strategy: IceMitigationStrategy,
    pub confidence: f64,
    pub raw_data_points: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IceType {
    Rime,
    Glaze,
    Mixed,
    Frost,
    WetSnow,
    SnowAccumulation,
    Unknown,
}

impl Default for IceType {
    fn default() -> Self {
        IceType::Unknown
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IceRiskLevel {
    None,
    Minimal,
    Low,
    Medium,
    Moderate,
    High,
    Severe,
    Critical,
}

impl Default for IceRiskLevel {
    fn default() -> Self {
        IceRiskLevel::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalIceComparison {
    pub same_period_last_year_thickness_mm: Option<f64>,
    pub historical_max_thickness_mm: f64,
    pub historical_avg_thickness_mm: f64,
    pub percentile_rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcePrediction {
    pub prediction_time: DateTime<Utc>,
    pub predicted_thickness_mm: f64,
    pub risk_level: IceRiskLevel,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceForecastPoint {
    pub timestamp: DateTime<Utc>,
    pub predicted_thickness_mm: f64,
    pub lower_bound_mm: f64,
    pub upper_bound_mm: f64,
    pub ambient_temp_c: f64,
    pub wind_speed_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceMitigationStrategy {
    pub immediate_actions: Vec<String>,
    pub short_term_actions: Vec<String>,
    pub long_term_actions: Vec<String>,
    pub recommended_speed_ms: Option<f64>,
    pub shutdown_recommended: bool,
    pub estimated_deicing_time_hours: Option<f64>,
    pub safe_ice_threshold_mm: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MitigationType {
    Heating,
    MechanicalVibration,
    ChemicalDeicing,
    ManualRemoval,
    Shutdown,
    SpeedReduction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyRating {
    VerySafe,
    Safe,
    Moderate,
    Risky,
    Dangerous,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceQuery {
    pub tower_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f64,
    pub value: f64,
    pub unit: String,
    pub contribution: String,
}
