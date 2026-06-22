use crate::{AppState, AppError, models::*, mq};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};

pub async fn analyze_ice_risk(
    state: AppState,
    tower_id: Uuid,
) -> Result<ice_detection::IceAnalysisResult, AppError> {
    let end_time = Utc::now();
    let start_time = end_time - Duration::minutes(30);

    let ice_data = state.clickhouse_client.query_ice_detection_data(
        &ice_detection::IceQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            page: Some(0),
            page_size: Some(1000),
        },
    ).await?;

    let vibration_data = state.clickhouse_client.query_vibration_data(
        &vibration::VibrationQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            frequency_range: None,
            page: Some(0),
            page_size: Some(1000),
        },
    ).await?;

    let wind_data = state.clickhouse_client.query_wind_speed_data(
        &vibration::VibrationQuery {
            tower_id: Some(tower_id),
            start_time: Some(start_time),
            end_time: Some(end_time),
            frequency_range: None,
            page: Some(0),
            page_size: Some(1000),
        },
    ).await?;

    let (avg_ice_thickness, max_ice_thickness) = if !ice_data.is_empty() {
        let avg = ice_data.iter().map(|d| d.ice_thickness_mm).sum::<f64>() / ice_data.len() as f64;
        let max = ice_data.iter().map(|d| d.ice_thickness_mm).fold(0.0, f64::max);
        (avg, max)
    } else {
        (0.0, 0.0)
    };

    let (avg_vibration, max_vibration) = if !vibration_data.is_empty() {
        let avg = vibration_data.iter().map(|d| d.velocity_mm_s).sum::<f64>() / vibration_data.len() as f64;
        let max = vibration_data.iter().map(|d| d.velocity_mm_s).fold(0.0, f64::max);
        (avg, max)
    } else {
        (0.0, 0.0)
    };

    let (avg_wind, max_wind) = if !wind_data.is_empty() {
        let avg = wind_data.iter().map(|d| d.velocity_mm_s).sum::<f64>() / wind_data.len() as f64;
        let max = wind_data.iter().map(|d| d.velocity_mm_s).fold(0.0, f64::max);
        (avg, max)
    } else {
        (0.0, 0.0)
    };

    let temp = ice_data.first()
        .map(|d| d.ambient_temp_c)
        .unwrap_or(0.0);

    let humidity = ice_data.first()
        .and_then(|d| d.humidity_percent)
        .unwrap_or(0.0);

    let (risk_level, risk_score) = calculate_ice_risk_score(
        avg_ice_thickness,
        avg_vibration,
        avg_wind,
        temp,
        humidity,
    );

    let ice_type = classify_ice_type(temp, humidity, avg_wind);
    let growth_rate = calculate_ice_growth_rate(&ice_data);
    let load_increase = estimate_load_increase(max_ice_thickness);
    let predictions = generate_ice_predictions(
        avg_ice_thickness,
        growth_rate,
        temp,
        humidity,
        avg_wind,
    );

    let mitigation = generate_mitigation_strategy(
        risk_level,
        max_ice_thickness,
        avg_wind,
    );

    let result = ice_detection::IceAnalysisResult {
        id: Uuid::new_v4(),
        tower_id,
        analysis_time: end_time,
        time_window_start: start_time,
        time_window_end: end_time,
        avg_ice_thickness_mm: avg_ice_thickness,
        max_ice_thickness_mm: max_ice_thickness,
        ice_type: Some(ice_type),
        ice_density_kg_m3: Some(900.0),
        temperature_c: temp,
        wind_speed_ms: avg_wind,
        humidity_percent: humidity,
        vibration_level_mm_s: avg_vibration,
        vibration_max_mm_s: max_vibration,
        load_increase_percent: load_increase,
        growth_rate_mm_h: growth_rate,
        risk_level,
        risk_score,
        contributing_factors: generate_contributing_factors(
            avg_ice_thickness,
            avg_vibration,
            avg_wind,
            temp,
            humidity,
        ),
        predictions,
        mitigation_strategy: mitigation,
        confidence: calculate_confidence(&ice_data, &vibration_data, &wind_data),
        raw_data_points: (ice_data.len() as u32),
    };

    state.clickhouse_client.insert_ice_analysis_result(&result).await?;

    if risk_level == ice_detection::IceRiskLevel::Critical || risk_level == ice_detection::IceRiskLevel::High {
        warn!("High ice risk detected for tower {}: score={:.2}, level={:?}",
            tower_id, risk_score, risk_level);

        let alert = alert::AlertMessage {
            id: Uuid::new_v4(),
            alert_type: alert::AlertType::IceDetection,
            severity: match risk_level {
                ice_detection::IceRiskLevel::Critical => alert::AlertSeverity::Critical,
                ice_detection::IceRiskLevel::High => alert::AlertSeverity::High,
                ice_detection::IceRiskLevel::Medium => alert::AlertSeverity::Medium,
                _ => alert::AlertSeverity::Low,
            },
            tower_id: Some(tower_id),
            sensor_id: None,
            title: format!("覆冰风险告警 - {:.1}mm", max_ice_thickness),
            message: format!("检测到塔架覆冰厚度{:.1}mm，风险等级{:?}，风险评分{:.2}",
                max_ice_thickness, risk_level, risk_score),
            timestamp: end_time,
            data: serde_json::to_value(&result)?,
            acknowledged: false,
            acknowledged_by: None,
            acknowledged_at: None,
        };

        mq::publisher::publish_alert_triggered(&state, &alert).await?;
    }

    Ok(result)
}

fn calculate_ice_risk_score(
    ice_thickness: f64,
    vibration: f64,
    wind_speed: f64,
    temperature: f64,
    humidity: f64,
) -> (ice_detection::IceRiskLevel, f64) {
    let mut score = 0.0;

    if ice_thickness > 15.0 { score += 40.0; }
    else if ice_thickness > 10.0 { score += 30.0; }
    else if ice_thickness > 5.0 { score += 20.0; }
    else if ice_thickness > 2.0 { score += 10.0; }

    if vibration > 5.0 { score += 25.0; }
    else if vibration > 3.0 { score += 15.0; }
    else if vibration > 1.5 { score += 8.0; }

    if wind_speed > 20.0 { score += 20.0; }
    else if wind_speed > 15.0 { score += 12.0; }
    else if wind_speed > 10.0 { score += 6.0; }

    if temperature < -5.0 { score += 10.0; }
    else if temperature < 0.0 { score += 5.0; }

    if humidity > 90.0 { score += 5.0; }

    let level = match score {
        s if s >= 70.0 => ice_detection::IceRiskLevel::Critical,
        s if s >= 50.0 => ice_detection::IceRiskLevel::High,
        s if s >= 30.0 => ice_detection::IceRiskLevel::Medium,
        s if s >= 10.0 => ice_detection::IceRiskLevel::Low,
        _ => ice_detection::IceRiskLevel::None,
    };

    (level, score)
}

fn classify_ice_type(
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
) -> ice_detection::IceType {
    if temperature < -10.0 && humidity > 80.0 && wind_speed > 15.0 {
        ice_detection::IceType::Rime
    } else if temperature < 0.0 && temperature > -5.0 && humidity > 90.0 {
        ice_detection::IceType::Glaze
    } else if temperature < -15.0 {
        ice_detection::IceType::SnowAccumulation
    } else if wind_speed > 20.0 {
        ice_detection::IceType::WetSnow
    } else {
        ice_detection::IceType::Mixed
    }
}

fn calculate_ice_growth_rate(ice_data: &[ice_detection::IceDetectionData]) -> f64 {
    if ice_data.len() < 2 {
        return 0.0;
    }

    let first = ice_data.first().unwrap();
    let last = ice_data.last().unwrap();

    let time_diff = last.timestamp.signed_duration_since(first.timestamp);
    let hours = time_diff.num_seconds() as f64 / 3600.0;

    if hours <= 0.0 {
        return 0.0;
    }

    (last.ice_thickness_mm - first.ice_thickness_mm) / hours
}

fn estimate_load_increase(ice_thickness: f64) -> f64 {
    if ice_thickness <= 0.0 {
        return 0.0;
    }

    let density = 900.0;
    let cable_diameter = 0.03;
    let cable_length = 100.0;

    let area = std::f64::consts::PI * ((cable_diameter / 2.0 + ice_thickness / 1000.0).powi(2) - (cable_diameter / 2.0).powi(2));
    let volume = area * cable_length;
    let mass = volume * density;
    let original_load = 50000.0;

    (mass / original_load) * 100.0
}

fn generate_ice_predictions(
    current_thickness: f64,
    growth_rate: f64,
    temperature: f64,
    humidity: f64,
    wind_speed: f64,
) -> Vec<ice_detection::IcePrediction> {
    let mut predictions = Vec::new();
    let now = Utc::now();

    let future_growth_rate = if temperature < -5.0 && humidity > 80.0 && wind_speed > 10.0 {
        growth_rate * 1.5
    } else if temperature < 0.0 {
        growth_rate * 1.2
    } else {
        growth_rate * 0.5
    };

    for hours in [1, 3, 6, 12, 24] {
        let predicted_thickness = (current_thickness + future_growth_rate * hours as f64).max(0.0);
        let (risk_level, _) = calculate_ice_risk_score(predicted_thickness, 0.0, wind_speed, temperature, humidity);

        predictions.push(ice_detection::IcePrediction {
            prediction_time: now + Duration::hours(hours as i64),
            predicted_thickness_mm: predicted_thickness,
            risk_level,
            confidence: 100.0 - (hours as f64) * 2.0,
        });
    }

    predictions
}

fn generate_mitigation_strategy(
    risk_level: ice_detection::IceRiskLevel,
    ice_thickness: f64,
    wind_speed: f64,
) -> ice_detection::IceMitigationStrategy {
    let mut immediate_actions = Vec::new();
    let mut short_term_actions = Vec::new();
    let mut long_term_actions = Vec::new();

    match risk_level {
        ice_detection::IceRiskLevel::Critical => {
            immediate_actions.push("立即启动停运程序，疏散索道乘客".to_string());
            immediate_actions.push("通知所有运维人员进入紧急状态".to_string());
            immediate_actions.push("启动加热除冰装置（如配备）".to_string());

            short_term_actions.push("安排紧急巡检，评估塔架结构完整性".to_string());
            short_term_actions.push("增加传感器采样频率".to_string());

            long_term_actions.push("考虑安装主动除冰系统".to_string());
        }
        ice_detection::IceRiskLevel::High => {
            immediate_actions.push("准备停运预案，密切关注覆冰发展".to_string());
            immediate_actions.push("通知值班人员加强监控".to_string());
            immediate_actions.push("启动加热除冰装置（如配备）".to_string());

            short_term_actions.push("安排预防性巡检".to_string());
            short_term_actions.push("准备除冰设备".to_string());

            long_term_actions.push("评估塔架抗冰载能力".to_string());
        }
        ice_detection::IceRiskLevel::Medium => {
            immediate_actions.push("加强监控，提高告警敏感度".to_string());

            short_term_actions.push("安排常规巡检".to_string());

            long_term_actions.push("优化除冰策略".to_string());
        }
        ice_detection::IceRiskLevel::Low => {
            immediate_actions.push("正常监控".to_string());
        }
        _ => {}
    }

    ice_detection::IceMitigationStrategy {
        immediate_actions,
        short_term_actions,
        long_term_actions,
        recommended_speed_ms: if risk_level >= ice_detection::IceRiskLevel::Medium {
            Some(5.0)
        } else {
            None
        },
        shutdown_recommended: risk_level >= ice_detection::IceRiskLevel::High,
        estimated_deicing_time_hours: if ice_thickness > 10.0 {
            Some(ice_thickness / 2.0)
        } else {
            None
        },
        safe_ice_threshold_mm: Some(10.0),
    }
}

fn generate_contributing_factors(
    ice_thickness: f64,
    vibration: f64,
    wind_speed: f64,
    temperature: f64,
    humidity: f64,
) -> Vec<ice_detection::RiskFactor> {
    let mut factors = Vec::new();

    if ice_thickness > 5.0 {
        factors.push(ice_detection::RiskFactor {
            name: "覆冰厚度".to_string(),
            value: ice_thickness,
            unit: "mm".to_string(),
            weight: 0.4,
            contribution: if ice_thickness > 10.0 { "严重" } else { "中等" }.to_string(),
        });
    }

    if vibration > 3.0 {
        factors.push(ice_detection::RiskFactor {
            name: "振动水平".to_string(),
            value: vibration,
            unit: "mm/s".to_string(),
            weight: 0.3,
            contribution: if vibration > 5.0 { "严重" } else { "中等" }.to_string(),
        });
    }

    if wind_speed > 15.0 {
        factors.push(ice_detection::RiskFactor {
            name: "风速".to_string(),
            value: wind_speed,
            unit: "m/s".to_string(),
            weight: 0.2,
            contribution: if wind_speed > 20.0 { "严重" } else { "中等" }.to_string(),
        });
    }

    if temperature < 0.0 {
        factors.push(ice_detection::RiskFactor {
            name: "环境温度".to_string(),
            value: temperature,
            unit: "°C".to_string(),
            weight: 0.05,
            contribution: "低".to_string(),
        });
    }

    if humidity > 85.0 {
        factors.push(ice_detection::RiskFactor {
            name: "空气湿度".to_string(),
            value: humidity,
            unit: "%".to_string(),
            weight: 0.05,
            contribution: "高".to_string(),
        });
    }

    factors
}

fn calculate_confidence(
    ice_data: &[ice_detection::IceDetectionData],
    vibration_data: &[vibration::VibrationData],
    wind_data: &[vibration::VibrationData],
) -> f64 {
    let mut score = 0.0;
    let mut weight = 0.0;

    if !ice_data.is_empty() {
        score += (ice_data.len().min(100) as f64 / 100.0) * 0.4;
        weight += 0.4;
    }

    if !vibration_data.is_empty() {
        score += (vibration_data.len().min(100) as f64 / 100.0) * 0.3;
        weight += 0.3;
    }

    if !wind_data.is_empty() {
        score += (wind_data.len().min(100) as f64 / 100.0) * 0.3;
        weight += 0.3;
    }

    if weight == 0.0 {
        0.0
    } else {
        (score / weight) * 100.0
    }
}
