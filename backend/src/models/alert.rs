use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub title: String,
    pub description: String,
    pub triggered_by: AlertTrigger,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub unit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
    pub video_verification_id: Option<Uuid>,
    pub related_data_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMessage {
    pub alert_id: Uuid,
    pub tower_id: Uuid,
    pub tower_name: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub actual_value: Option<f64>,
    pub threshold_value: Option<f64>,
    pub unit: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub risk_level: super::tower::RiskLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    VibrationHigh,
    VibrationAnomaly,
    WindSpeedHigh,
    WindGustHigh,
    IceThicknessHigh,
    IceAccumulationFast,
    IceLoadCritical,
    TemperatureExtreme,
    SensorFault,
    CommunicationError,
    PowerFailure,
    VideoVerificationRequired,
    ShutdownRecommended,
    ShutdownRequired,
    SystemError,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    High,
    Critical,
    Emergency,
}

impl Default for AlertSeverity {
    fn default() -> Self {
        AlertSeverity::Info
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Open,
    Acknowledged,
    InProgress,
    Resolved,
    Closed,
    FalseAlarm,
}

impl Default for AlertStatus {
    fn default() -> Self {
        AlertStatus::Open
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AlertTrigger {
    SensorReading {
        sensor_id: Uuid,
        reading_id: Uuid,
    },
    AnalysisResult {
        analysis_id: Uuid,
        analysis_type: String,
    },
    VideoVerification {
        verification_id: Uuid,
        confidence: f64,
    },
    Manual {
        user_id: Uuid,
        reason: String,
    },
    System {
        component: String,
        error_code: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertRequest {
    pub tower_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub triggered_by: AlertTrigger,
    pub threshold_value: Option<f64>,
    pub actual_value: Option<f64>,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub user_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveAlertRequest {
    pub user_id: Uuid,
    pub resolution_notes: String,
    pub new_status: AlertStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertQuery {
    pub tower_id: Option<Uuid>,
    pub alert_type: Option<AlertType>,
    pub severity: Option<AlertSeverity>,
    pub status: Option<AlertStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_open: u32,
    pub total_acknowledged: u32,
    pub by_severity: std::collections::HashMap<AlertSeverity, u32>,
    pub by_type: std::collections::HashMap<AlertType, u32>,
    pub last_24h_count: u32,
    pub unresolved_critical: u32,
}
