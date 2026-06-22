use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub temperature_c: f64,
    pub humidity_percent: f64,
    pub pressure_hpa: f64,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: f64,
    pub wind_gust_ms: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub precipitation_type: PrecipitationType,
    pub visibility_m: Option<f64>,
    pub cloud_cover_percent: Option<f64>,
    pub solar_radiation_w_m2: Option<f64>,
    pub dew_point_c: Option<f64>,
    pub wind_chill_c: Option<f64>,
    pub source: WeatherDataSource,
    pub quality: super::sensor::ReadingQuality,
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
    Drizzle,
    Mixed,
}

impl Default for PrecipitationType {
    fn default() -> Self {
        PrecipitationType::None
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeatherDataSource {
    Sensor,
    WeatherAPI,
    Forecast,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherForecast {
    pub id: Uuid,
    pub tower_id: Option<Uuid>,
    pub forecast_time: DateTime<Utc>,
    pub forecast_hours: u32,
    pub hourly_forecast: Vec<HourlyForecast>,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyForecast {
    pub timestamp: DateTime<Utc>,
    pub temperature_c: f64,
    pub humidity_percent: f64,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: f64,
    pub wind_gust_ms: Option<f64>,
    pub precipitation_probability_percent: f64,
    pub precipitation_mm: f64,
    pub precipitation_type: PrecipitationType,
    pub cloud_cover_percent: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeatherRiskType {
    Ice,
    HighWind,
    ExtremeTemperature,
    HeavyPrecipitation,
    General,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationRecommendation {
    pub risk_type: WeatherRiskType,
    pub priority: RecommendationPriority,
    pub action: String,
    pub estimated_cost: f64,
    pub effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub operational_impact: String,
    pub structural_impact: String,
    pub maintenance_impact: String,
    pub estimated_cost_increase_percent: f64,
    pub estimated_downtime_hours: f64,
    pub passenger_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastSummary {
    pub forecast_period_days: u32,
    pub forecast_time: DateTime<Utc>,
    pub avg_temperature_c: f64,
    pub min_temperature_c: f64,
    pub max_temperature_c: f64,
    pub avg_wind_speed_ms: f64,
    pub max_wind_speed_ms: f64,
    pub precipitation_probability: f64,
    pub expected_ice_days: u32,
    pub expected_high_wind_days: u32,
    pub expected_storm_days: u32,
    pub overall_forecast_risk: RiskLevel,
    pub summary_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherImpactAnalysis {
    pub id: Uuid,
    pub tower_id: Option<Uuid>,
    pub analysis_time: DateTime<Utc>,
    pub analysis_period_start: DateTime<Utc>,
    pub analysis_period_end: DateTime<Utc>,
    pub data_points_count: u32,
    pub weather_summary: WeatherSummary,
    pub alerts: Vec<WeatherAlert>,
    pub overall_risk: RiskLevel,
    pub impact_rating: ImpactRating,
    pub risk_by_type: Vec<(WeatherRiskType, RiskLevel, f64)>,
    pub impact_assessment: ImpactAssessment,
    pub mitigation_recommendations: Vec<MitigationRecommendation>,
    pub forecast: Option<ForecastSummary>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherSummary {
    pub avg_temperature_c: f64,
    pub min_temperature_c: f64,
    pub max_temperature_c: f64,
    pub avg_humidity_percent: f64,
    pub avg_wind_speed_ms: f64,
    pub max_wind_speed_ms: f64,
    pub avg_wind_gust_ms: Option<f64>,
    pub dominant_wind_direction: f64,
    pub total_precipitation_mm: f64,
    pub precipitation_type: PrecipitationType,
    pub min_visibility_m: Option<f64>,
    pub days_with_ice_risk: u32,
    pub days_with_high_wind: u32,
    pub days_with_extreme_temp: u32,
    pub days_with_precipitation: u32,
}

impl Default for WeatherSummary {
    fn default() -> Self {
        WeatherSummary {
            avg_temperature_c: 0.0,
            min_temperature_c: 0.0,
            max_temperature_c: 0.0,
            avg_humidity_percent: 0.0,
            avg_wind_speed_ms: 0.0,
            max_wind_speed_ms: 0.0,
            avg_wind_gust_ms: None,
            dominant_wind_direction: 0.0,
            total_precipitation_mm: 0.0,
            precipitation_type: PrecipitationType::None,
            min_visibility_m: None,
            days_with_ice_risk: 0,
            days_with_high_wind: 0,
            days_with_extreme_temp: 0,
            days_with_precipitation: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceRiskAssessment {
    pub risk_level: RiskLevel,
    pub probability_percent: f64,
    pub expected_thickness_mm: f64,
    pub accumulation_rate_mm_h: f64,
    pub contributing_factors: Vec<String>,
    pub critical_temperature_c: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindRiskAssessment {
    pub risk_level: RiskLevel,
    pub probability_percent: f64,
    pub expected_max_speed_ms: f64,
    pub expected_gust_ms: f64,
    pub sustained_duration_minutes: i64,
    pub wind_load_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibrationRiskAssessment {
    pub risk_level: RiskLevel,
    pub probability_percent: f64,
    pub expected_velocity_mm_s: f64,
    pub resonant_frequency_risk: bool,
    pub contributing_factors: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
    Extreme,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::None
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImpactRating {
    Negligible,
    Minor,
    Moderate,
    Major,
    Significant,
    Severe,
    Extreme,
}

impl Default for ImpactRating {
    fn default() -> Self {
        ImpactRating::Negligible
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherHistoricalComparison {
    pub same_period_last_year: WeatherSummary,
    pub historical_max_wind_speed_ms: f64,
    pub historical_max_ice_thickness_mm: f64,
    pub percentile_rank_temperature: f64,
    pub percentile_rank_wind: f64,
    pub percentile_rank_precipitation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherAlert {
    pub id: Uuid,
    pub tower_id: Option<Uuid>,
    pub alert_type: WeatherAlertType,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
    pub title: String,
    pub headline: String,
    pub message: String,
    pub description: String,
    pub data: Option<serde_json::Value>,
    pub effective_start: DateTime<Utc>,
    pub effective_end: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub affected_area: String,
    pub response_type: ResponseType,
    pub certainty: Certainty,
    pub urgency: Urgency,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeatherAlertType {
    WindWarning,
    IceWarning,
    IceStormWarning,
    BlizzardWarning,
    FreezingRainWarning,
    ExtremeColdWarning,
    ThunderstormWarning,
    HeavySnowWarning,
    AvalancheWarning,
    FrostWarning,
    DenseFogWarning,
    HighWindWarning,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Low,
    Minor,
    Moderate,
    Severe,
    Extreme,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseType {
    Monitor,
    Prepare,
    Evacuate,
    Shelter,
    ExecuteShutdown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Certainty {
    Observed,
    Likely,
    Possible,
    Unlikely,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Immediate,
    Expected,
    Future,
    Past,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherQuery {
    pub tower_id: Option<Uuid>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_temperature: Option<f64>,
    pub max_temperature: Option<f64>,
    pub min_wind_speed: Option<f64>,
    pub max_wind_speed: Option<f64>,
    pub precipitation_type: Option<PrecipitationType>,
}
