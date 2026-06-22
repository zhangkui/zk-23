use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindSpeedData {
    pub id: Uuid,
    pub tower_id: Uuid,
    pub sensor_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: f64,
    pub gust_speed_ms: Option<f64>,
    pub temperature: Option<f64>,
    pub quality: super::sensor::ReadingQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindSpeedReading {
    pub sensor_id: Uuid,
    pub tower_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub wind_speed_ms: f64,
    pub wind_direction_deg: f64,
    pub gust_speed_ms: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindAnalysisResult {
    pub tower_id: Uuid,
    pub analysis_time: DateTime<Utc>,
    pub time_window_start: DateTime<Utc>,
    pub time_window_end: DateTime<Utc>,
    pub avg_speed: f64,
    pub max_speed: f64,
    pub min_speed: f64,
    pub std_speed: f64,
    pub avg_direction: f64,
    pub direction_variance: f64,
    pub max_gust: f64,
    pub wind_load_factor: f64,
    pub risk_assessment: WindRiskLevel,
    pub sustained_high_speed_duration_min: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindRiskLevel {
    Low,
    Moderate,
    High,
    Extreme,
}

impl Default for WindRiskLevel {
    fn default() -> Self {
        WindRiskLevel::Low
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaufortScale {
    pub force: u8,
    pub name: String,
    pub min_speed_ms: f64,
    pub max_speed_ms: f64,
    pub description: String,
    pub impact_on_cableway: String,
}

pub fn beaufort_scale(speed_ms: f64) -> BeaufortScale {
    match speed_ms {
        s if s < 0.3 => BeaufortScale {
            force: 0,
            name: "Calm".to_string(),
            min_speed_ms: 0.0,
            max_speed_ms: 0.2,
            description: "无风".to_string(),
            impact_on_cableway: "正常运行".to_string(),
        },
        s if s < 1.6 => BeaufortScale {
            force: 1,
            name: "Light air".to_string(),
            min_speed_ms: 0.3,
            max_speed_ms: 1.5,
            description: "软风".to_string(),
            impact_on_cableway: "正常运行".to_string(),
        },
        s if s < 3.4 => BeaufortScale {
            force: 2,
            name: "Light breeze".to_string(),
            min_speed_ms: 1.6,
            max_speed_ms: 3.3,
            description: "轻风".to_string(),
            impact_on_cableway: "正常运行".to_string(),
        },
        s if s < 5.5 => BeaufortScale {
            force: 3,
            name: "Gentle breeze".to_string(),
            min_speed_ms: 3.4,
            max_speed_ms: 5.4,
            description: "微风".to_string(),
            impact_on_cableway: "正常运行".to_string(),
        },
        s if s < 8.0 => BeaufortScale {
            force: 4,
            name: "Moderate breeze".to_string(),
            min_speed_ms: 5.5,
            max_speed_ms: 7.9,
            description: "和风".to_string(),
            impact_on_cableway: "正常运行，注意观察".to_string(),
        },
        s if s < 10.8 => BeaufortScale {
            force: 5,
            name: "Fresh breeze".to_string(),
            min_speed_ms: 8.0,
            max_speed_ms: 10.7,
            description: "清劲风".to_string(),
            impact_on_cableway: "正常运行，加强监测".to_string(),
        },
        s if s < 13.9 => BeaufortScale {
            force: 6,
            name: "Strong breeze".to_string(),
            min_speed_ms: 10.8,
            max_speed_ms: 13.8,
            description: "强风".to_string(),
            impact_on_cableway: "降低速度运行".to_string(),
        },
        s if s < 17.2 => BeaufortScale {
            force: 7,
            name: "Near gale".to_string(),
            min_speed_ms: 13.9,
            max_speed_ms: 17.1,
            description: "疾风".to_string(),
            impact_on_cableway: "准备停运".to_string(),
        },
        s if s < 20.8 => BeaufortScale {
            force: 8,
            name: "Gale".to_string(),
            min_speed_ms: 17.2,
            max_speed_ms: 20.7,
            description: "大风".to_string(),
            impact_on_cableway: "建议停运".to_string(),
        },
        s if s < 24.5 => BeaufortScale {
            force: 9,
            name: "Strong gale".to_string(),
            min_speed_ms: 20.8,
            max_speed_ms: 24.4,
            description: "烈风".to_string(),
            impact_on_cableway: "必须停运".to_string(),
        },
        s if s < 28.5 => BeaufortScale {
            force: 10,
            name: "Storm".to_string(),
            min_speed_ms: 24.5,
            max_speed_ms: 28.4,
            description: "狂风".to_string(),
            impact_on_cableway: "停运，紧急加固措施".to_string(),
        },
        s if s < 32.7 => BeaufortScale {
            force: 11,
            name: "Violent storm".to_string(),
            min_speed_ms: 28.5,
            max_speed_ms: 32.6,
            description: "暴风".to_string(),
            impact_on_cableway: "停运，应急响应".to_string(),
        },
        _ => BeaufortScale {
            force: 12,
            name: "Hurricane".to_string(),
            min_speed_ms: 32.7,
            max_speed_ms: 100.0,
            description: "飓风".to_string(),
            impact_on_cableway: "停运，紧急疏散".to_string(),
        },
    }
}
