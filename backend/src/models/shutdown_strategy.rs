use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownStrategy {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub strategy_type: ShutdownType,
    pub status: StrategyStatus,
    pub severity: StrategySeverity,
    pub title: String,
    pub description: String,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub action_steps: Vec<ActionStep>,
    pub estimated_duration_minutes: i64,
    pub affected_area: String,
    pub safety_measures: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub executed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub related_alert_ids: Vec<Uuid>,
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownType {
    Preemptive,
    Emergency,
    Scheduled,
    Manual,
    Recovery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StrategyStatus {
    Draft,
    PendingApproval,
    Approved,
    Triggered,
    Executing,
    Completed,
    Cancelled,
    Expired,
}

impl Default for StrategyStatus {
    fn default() -> Self {
        StrategyStatus::Draft
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum StrategySeverity {
    Advisory,
    Watch,
    Warning,
    Severe,
    Extreme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub id: Uuid,
    pub condition_type: ConditionType,
    pub metric: String,
    pub operator: ConditionOperator,
    pub threshold: f64,
    pub unit: String,
    pub duration_minutes: Option<i64>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    VibrationVelocity,
    VibrationFrequency,
    WindSpeed,
    WindGust,
    IceThickness,
    IceAccumulationRate,
    Temperature,
    CombinedRisk,
    VideoConfirmation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStep {
    pub id: Uuid,
    pub step_number: u32,
    pub action: String,
    pub responsible_role: String,
    pub estimated_duration_minutes: i64,
    pub is_critical: bool,
    pub prerequisites: Vec<Uuid>,
    pub completion_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStrategyRequest {
    pub tower_id: Uuid,
    pub strategy_type: ShutdownType,
    pub severity: StrategySeverity,
    pub title: String,
    pub description: String,
    pub trigger_conditions: Vec<TriggerCondition>,
    pub action_steps: Vec<ActionStep>,
    pub estimated_duration_minutes: i64,
    pub affected_area: String,
    pub safety_measures: Vec<String>,
    pub auto_approve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteStrategyRequest {
    pub strategy_id: Uuid,
    pub executed_by: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEvaluationResult {
    pub strategy_id: Uuid,
    pub tower_id: Uuid,
    pub evaluation_time: DateTime<Utc>,
    pub should_trigger: bool,
    pub trigger_reason: Option<String>,
    pub met_conditions: Vec<Uuid>,
    pub unmet_conditions: Vec<Uuid>,
    pub overall_risk_score: f64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownLog {
    pub id: Uuid,
    pub strategy_id: Uuid,
    pub tower_id: Uuid,
    pub shutdown_type: ShutdownType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i64>,
    pub reason: String,
    pub executed_by: Uuid,
    pub passengers_affected: Option<u32>,
    pub economic_impact: Option<f64>,
    pub weather_conditions: WeatherConditions,
    pub safety_incidents: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConditions {
    pub temperature_c: f64,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: f64,
    pub ice_thickness_mm: Option<f64>,
    pub visibility_m: Option<f64>,
    pub precipitation_type: Option<super::ice_detection::PrecipitationType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartPlan {
    pub id: Uuid,
    pub shutdown_log_id: Uuid,
    pub tower_id: Uuid,
    pub planned_restart_time: DateTime<Utc>,
    pub required_checks: Vec<RestartCheck>,
    pub status: RestartStatus,
    pub approved_by: Option<Uuid>,
    pub actual_restart_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestartCheck {
    pub id: Uuid,
    pub check_name: String,
    pub description: String,
    pub is_completed: bool,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestartStatus {
    Pending,
    InProgress,
    Ready,
    Approved,
    Completed,
    Cancelled,
}

impl Default for RestartStatus {
    fn default() -> Self {
        RestartStatus::Pending
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum UrgencyLevel {
    None,
    Low,
    Medium,
    High,
    Immediate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShutdownStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for RiskLevel {
    fn default() -> Self {
        RiskLevel::None
    }
}
