use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub device_id: String,
    pub name: String,
    pub location: String,
    pub mount_position: String,
    pub camera_type: CameraType,
    pub status: CameraStatus,
    pub rtsp_url: Option<String>,
    pub http_url: Option<String>,
    pub resolution: String,
    pub fps: u32,
    pub has_ai_analysis: bool,
    pub ai_model_version: Option<String>,
    pub last_online: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CameraType {
    PTZ,
    Fixed,
    Thermal,
    Dome,
    Bullet,
    Fisheye,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CameraStatus {
    Online,
    Offline,
    Recording,
    Maintenance,
    Faulty,
    Disabled,
}

impl Default for CameraStatus {
    fn default() -> Self {
        CameraStatus::Offline
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoVerificationRequest {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub camera_id: Uuid,
    pub alert_id: Option<Uuid>,
    pub request_type: VerificationType,
    pub status: VerificationStatus,
    pub priority: super::inspection::Priority,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub description: String,
    pub items_to_verify: Vec<String>,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationType {
    IcePresence,
    IceThickness,
    StructuralDamage,
    CableCondition,
    WindEffect,
    GeneralInspection,
    AlertConfirmation,
    IncidentReview,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Pending,
    InProgress,
    Completed,
    Expired,
    Cancelled,
}

impl Default for VerificationStatus {
    fn default() -> Self {
        VerificationStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoVerificationResult {
    pub id: Uuid,
    pub request_id: Uuid,
    pub tower_id: Uuid,
    pub camera_id: Uuid,
    pub verified_by: Option<Uuid>,
    pub verification_method: VerificationMethod,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub overall_findings: Vec<String>,
    pub ice_verification: Option<IceVerification>,
    pub damage_verification: Option<DamageVerification>,
    pub status_verification: Option<StatusVerification>,
    pub ai_confidence: Option<f64>,
    pub human_review_required: bool,
    pub human_reviewed: bool,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub attachments: Vec<VideoAttachment>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationMethod {
    AIOnly,
    HumanOnly,
    AIWithHumanReview,
    LiveStream,
    Snapshot,
    RecordedVideo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceVerification {
    pub ice_present: bool,
    pub estimated_thickness_mm: f64,
    pub thickness_confidence: f64,
    pub ice_type: super::ice_detection::IceType,
    pub affected_areas: Vec<String>,
    pub sensor_correlation: SensorCorrelation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageVerification {
    pub damage_detected: bool,
    pub damage_type: String,
    pub damage_severity: super::inspection::FindingSeverity,
    pub damage_location: String,
    pub damage_description: String,
    pub requires_immediate_action: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusVerification {
    pub structural_status: StructuralStatus,
    pub cable_tension_status: TensionStatus,
    pub overall_status: super::tower::TowerStatus,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StructuralStatus {
    Normal,
    MinorConcern,
    MajorConcern,
    CriticalDamage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TensionStatus {
    Normal,
    SlightlyLoose,
    Loose,
    OverTensioned,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorCorrelation {
    pub sensor_thickness_mm: f64,
    pub video_thickness_mm: f64,
    pub difference_mm: f64,
    pub correlation_coefficient: f64,
    pub sensor_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAttachment {
    pub id: Uuid,
    pub attachment_type: AttachmentType,
    pub file_name: String,
    pub file_size_bytes: i64,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub description: Option<String>,
    pub ai_annotations: Vec<AIAnnotation>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    Snapshot,
    VideoClip,
    ThermalImage,
    Panorama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnnotation {
    pub id: Uuid,
    pub annotation_type: String,
    pub label: String,
    pub confidence: f64,
    pub bounding_box: BoundingBox,
    pub mask_coordinates: Option<Vec<Coordinate>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub label: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisResult {
    pub analysis_type: AIAnalysisType,
    pub confidence: f64,
    pub detected: bool,
    pub description: String,
    pub bounding_boxes: Option<Vec<BoundingBox>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVerificationRequest {
    pub tower_id: Uuid,
    pub camera_id: Option<Uuid>,
    pub request_type: VerificationType,
    pub priority: super::inspection::Priority,
    pub requested_by: Uuid,
    pub description: String,
    pub items_to_verify: Vec<String>,
    pub alert_id: Option<Uuid>,
    pub scheduled_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStreamSession {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub tower_id: Uuid,
    pub user_id: Uuid,
    pub session_type: SessionType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i64>,
    pub stream_url: String,
    pub recording_url: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    ManualReview,
    AlertResponse,
    ScheduledInspection,
    EmergencyResponse,
    Training,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIAnalysisConfig {
    pub id: Uuid,
    pub camera_id: Uuid,
    pub ice_detection_enabled: bool,
    pub ice_detection_threshold_mm: f64,
    pub ice_detection_min_confidence: f64,
    pub damage_detection_enabled: bool,
    pub damage_detection_min_confidence: f64,
    pub human_detection_enabled: bool,
    pub auto_verification_trigger: bool,
    pub analysis_interval_seconds: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::str::FromStr for VerificationPriority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(VerificationPriority::Low),
            "medium" => Ok(VerificationPriority::Medium),
            "high" => Ok(VerificationPriority::High),
            "critical" => Ok(VerificationPriority::Critical),
            _ => Err(format!("Invalid priority: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AIAnalysisType {
    IceDetection,
    StructureInspection,
    DamageDetection,
    PersonDetection,
    CableInspection,
    GeneralMonitoring,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamType {
    RTSP,
    WebRTC,
    HLS,
    DASH,
    HTTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoQuery {
    pub tower_id: Option<Uuid>,
    pub camera_id: Option<Uuid>,
    pub verification_type: Option<VerificationType>,
    pub status: Option<VerificationStatus>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub human_reviewed: Option<bool>,
}
