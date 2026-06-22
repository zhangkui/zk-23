use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Tower {
    pub id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub code: String,
    pub location: Location,
    pub height_meters: f64,
    pub construction_date: Option<DateTime<Utc>>,
    pub status: TowerStatus,
    pub cable_line_id: Uuid,
    pub position_in_line: u32,
    pub max_load_kg: f64,
    pub last_inspection_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub sensors: Vec<SensorMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation_meters: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TowerStatus {
    Operational,
    Warning,
    Shutdown,
    Maintenance,
    Decommissioned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMount {
    pub sensor_id: Uuid,
    pub mount_position: String,
    pub mount_height_meters: f64,
    pub direction: Option<f64>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTowerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub code: String,
    pub location: Location,
    pub height_meters: f64,
    pub construction_date: Option<DateTime<Utc>>,
    pub cable_line_id: Uuid,
    pub position_in_line: u32,
    pub max_load_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTowerRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    pub status: Option<TowerStatus>,
    pub last_inspection_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerStatusResponse {
    pub tower_id: Uuid,
    pub tower_name: String,
    pub status: TowerStatus,
    pub vibration_level: f64,
    pub wind_speed: f64,
    pub ice_thickness: f64,
    pub risk_level: RiskLevel,
    pub last_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerListQuery {
    pub cable_line_id: Option<Uuid>,
    pub status: Option<TowerStatus>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
