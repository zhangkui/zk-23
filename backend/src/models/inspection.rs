use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRecord {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub inspection_type: InspectionType,
    pub status: InspectionStatus,
    pub inspector_id: Uuid,
    pub inspector_name: String,
    pub scheduled_date: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i64>,
    pub weather_conditions: super::shutdown_strategy::WeatherConditions,
    pub check_items: Vec<InspectionCheckItem>,
    pub overall_condition: OverallCondition,
    pub findings: Vec<InspectionFinding>,
    pub recommendations: Vec<String>,
    pub follow_up_required: bool,
    pub follow_up_date: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub attachments: Vec<Attachment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspectionType {
    Routine,
    AfterStorm,
    AfterShutdown,
    Complaint,
    PreventiveMaintenance,
    Emergency,
    Annual,
    Seasonal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspectionStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    Overdue,
}

impl Default for InspectionStatus {
    fn default() -> Self {
        InspectionStatus::Scheduled
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverallCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCheckItem {
    pub id: Uuid,
    pub category: String,
    pub item_name: String,
    pub description: String,
    pub is_checked: bool,
    pub condition: ItemCondition,
    pub notes: Option<String>,
    pub photo_url: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemCondition {
    NotApplicable,
    Good,
    MinorIssue,
    MajorIssue,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionFinding {
    pub id: Uuid,
    pub severity: FindingSeverity,
    pub category: String,
    pub description: String,
    pub location: String,
    pub estimated_repair_cost: Option<f64>,
    pub priority: Priority,
    pub requires_immediate_action: bool,
    pub status: FindingStatus,
    pub resolution_date: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FindingSeverity {
    Cosmetic,
    Minor,
    Moderate,
    Major,
    Critical,
    Safety,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
    Urgent,
    Immediate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MaintenancePriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::str::FromStr for MaintenancePriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(MaintenancePriority::Low),
            "medium" => Ok(MaintenancePriority::Medium),
            "high" => Ok(MaintenancePriority::High),
            "critical" => Ok(MaintenancePriority::Critical),
            _ => Err(format!("Invalid maintenance priority: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Monitor,
    Closed,
}

impl Default for FindingStatus {
    fn default() -> Self {
        FindingStatus::Open
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub file_name: String,
    pub file_type: String,
    pub file_size_bytes: i64,
    pub url: String,
    pub description: Option<String>,
    pub uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInspectionRequest {
    pub tower_id: Uuid,
    pub inspection_type: InspectionType,
    pub inspector_id: Uuid,
    pub inspector_name: String,
    pub scheduled_date: DateTime<Utc>,
    pub check_items: Vec<InspectionCheckItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInspectionRequest {
    pub status: Option<InspectionStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub overall_condition: Option<OverallCondition>,
    pub findings: Option<Vec<InspectionFinding>>,
    pub recommendations: Option<Vec<String>>,
    pub notes: Option<String>,
    pub weather_conditions: Option<super::shutdown_strategy::WeatherConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceTask {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub source_inspection_id: Option<Uuid>,
    pub source_finding_id: Option<Uuid>,
    pub task_type: String,
    pub description: String,
    pub priority: Priority,
    pub status: MaintenanceStatus,
    pub assigned_to: Option<Uuid>,
    pub scheduled_date: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_date: Option<DateTime<Utc>>,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub parts_used: Vec<String>,
    pub labor_hours: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceStatus {
    Pending,
    Scheduled,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
    Overdue,
}

impl Default for MaintenanceStatus {
    fn default() -> Self {
        MaintenanceStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionQuery {
    pub tower_id: Option<Uuid>,
    pub inspection_type: Option<InspectionType>,
    pub status: Option<InspectionStatus>,
    pub overall_condition: Option<OverallCondition>,
    pub inspector_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionStatistics {
    pub total_inspections: u32,
    pub by_type: std::collections::HashMap<InspectionType, u32>,
    pub by_status: std::collections::HashMap<InspectionStatus, u32>,
    pub by_condition: std::collections::HashMap<OverallCondition, u32>,
    pub average_duration_minutes: f64,
    pub open_findings: u32,
    pub critical_findings: u32,
    pub overdue_inspections: u32,
    pub last_30_days_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckCategory {
    Foundation,
    Structure,
    Cable,
    Sensor,
    Machinery,
    Electrical,
    Safety,
    General,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckResult {
    Pass,
    Warning,
    Fail,
    NotApplicable,
    Deferred,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingType {
    Corrosion,
    LooseConnection,
    StructuralDamage,
    Wear,
    Misalignment,
    ElectricalFault,
    SensorFault,
    SafetyHazard,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceType {
    Repair,
    Replacement,
    Lubrication,
    Adjustment,
    Calibration,
    Cleaning,
    Inspection,
    Upgrade,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionReport {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub generated_at: DateTime<Utc>,
    pub total_inspections: u32,
    pub total_findings: u32,
    pub critical_findings: u32,
    pub resolved_findings: u32,
    pub maintenance_tasks_created: u32,
    pub maintenance_tasks_completed: u32,
    pub maintenance_tasks_pending: u32,
    pub high_priority_tasks: u32,
    pub average_inspection_score: f64,
    pub recommendations: Vec<String>,
    pub next_inspection_date: DateTime<Utc>,
    pub prepared_by: Option<String>,
}
