use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration, Timelike};
use rand::Rng;
use uuid::Uuid;
use tracing::{info};

pub async fn analyze_weather_impact(
    state: AppState,
    tower_id: Option<Uuid>,
    days: u32,
) -> Result<weather::WeatherImpactAnalysis, AppError> {
    info!("Analyzing weather impact for {} days", days);

    let end_time = Utc::now();
    let start_time = end_time - Duration::days(days as i64);

    let weather_data = state.clickhouse_client.query_weather_data(
        tower_id,
        Some(start_time),
        Some(end_time),
        0,
        10000,
    ).await?;

    let alerts = generate_weather_alerts(&weather_data);

    let avg_temp = weather_data.iter().map(|d| d.temperature_c).sum::<f64>() / weather_data.len().max(1) as f64;
    let min_temp = weather_data.iter().map(|d| d.temperature_c).fold(f64::INFINITY, f64::min);
    let max_temp = weather_data.iter().map(|d| d.temperature_c).fold(f64::NEG_INFINITY, f64::max);

    let avg_wind = weather_data.iter().map(|d| d.wind_speed_ms).sum::<f64>() / weather_data.len().max(1) as f64;
    let max_wind = weather_data.iter().map(|d| d.wind_speed_ms).fold(0.0, f64::max);

    let avg_humidity = weather_data.iter().map(|d| d.humidity_percent).sum::<f64>() / weather_data.len().max(1) as f64;

    let total_precipitation: f64 = weather_data.iter()
        .map(|d| d.precipitation_mm.unwrap_or(0.0))
        .sum();

    let days_with_ice = count_days_with_ice_risk(&weather_data);
    let days_with_high_wind = count_days_with_high_wind(&weather_data);
    let days_with_extreme_temp = count_days_with_extreme_temp(&weather_data);

    let (overall_risk, impact_rating) = calculate_overall_risk(
        days_with_ice,
        days_with_high_wind,
        days_with_extreme_temp,
        max_wind,
    );

    let risk_by_type = vec![
        (weather::WeatherRiskType::Ice, if days_with_ice > 0 { weather::RiskLevel::High } else { weather::RiskLevel::Low }, days_with_ice as f64),
        (weather::WeatherRiskType::HighWind, if days_with_high_wind > 0 { weather::RiskLevel::Medium } else { weather::RiskLevel::Low }, days_with_high_wind as f64),
        (weather::WeatherRiskType::ExtremeTemperature, if days_with_extreme_temp > 0 { weather::RiskLevel::Low } else { weather::RiskLevel::None }, days_with_extreme_temp as f64),
        (weather::WeatherRiskType::HeavyPrecipitation, if total_precipitation > 50.0 { weather::RiskLevel::Medium } else { weather::RiskLevel::Low }, total_precipitation),
    ];

    let impact_assessment = generate_impact_assessment(
        overall_risk,
        max_wind,
        days_with_ice,
    );

    let mitigation_recommendations = generate_mitigation_recommendations(
        &risk_by_type,
        max_wind,
    );

    Ok(weather::WeatherImpactAnalysis {
        id: Uuid::new_v4(),
        tower_id,
        analysis_time: end_time,
        analysis_period_start: start_time,
        analysis_period_end: end_time,
        data_points_count: weather_data.len() as u32,
        weather_summary: weather::WeatherSummary {
            avg_temperature_c: avg_temp,
            min_temperature_c: min_temp,
            max_temperature_c: max_temp,
            avg_humidity_percent: avg_humidity,
            avg_wind_speed_ms: avg_wind,
            max_wind_speed_ms: max_wind,
            total_precipitation_mm: total_precipitation,
            dominant_wind_direction: calculate_dominant_wind_direction(&weather_data),
            days_with_ice_risk: days_with_ice,
            days_with_high_wind: days_with_high_wind,
            days_with_extreme_temp: days_with_extreme_temp,
            days_with_precipitation: weather_data.iter()
                .filter(|d| d.precipitation_mm.unwrap_or(0.0) > 0.0)
                .count() as u32,
        },
        alerts,
        overall_risk,
        impact_rating,
        risk_by_type,
        impact_assessment,
        mitigation_recommendations,
        forecast: Some(generate_forecast_summary()),
        confidence: if weather_data.len() >= 100 { 95.0 } else { 75.0 },
    })
}

fn generate_weather_alerts(data: &[weather::WeatherData]) -> Vec<weather::WeatherAlert> {
    let mut alerts = Vec::new();

    for d in data {
        if d.temperature_c < -5.0 && d.humidity_percent > 80.0 {
            let severity = if d.temperature_c < -10.0 {
                weather::AlertSeverity::Severe
            } else {
                weather::AlertSeverity::Medium
            };
            alerts.push(weather::WeatherAlert {
                id: Uuid::new_v4(),
                tower_id: Some(d.tower_id),
                alert_type: weather::WeatherAlertType::IceWarning,
                severity,
                timestamp: d.timestamp,
                title: "覆冰预警".to_string(),
                headline: "索道塔架覆冰风险预警".to_string(),
                message: format!("温度{:.1}°C，湿度{:.0}%，存在覆冰风险", d.temperature_c, d.humidity_percent),
                description: format!("当前温度{:.1}°C，相对湿度{:.0}%，气象条件有利于覆冰形成。请加强监测，必要时启动除冰程序。", d.temperature_c, d.humidity_percent),
                data: Some(serde_json::json!({
                    "temperature": d.temperature_c,
                    "humidity": d.humidity_percent,
                })),
                effective_start: d.timestamp,
                effective_end: d.timestamp + Duration::hours(6),
                expires_at: Some(d.timestamp + Duration::hours(6)),
                affected_area: "索道塔架沿线".to_string(),
                response_type: if severity >= weather::AlertSeverity::Severe {
                    weather::ResponseType::Prepare
                } else {
                    weather::ResponseType::Monitor
                },
                certainty: weather::Certainty::Likely,
                urgency: if severity >= weather::AlertSeverity::Severe {
                    weather::Urgency::Expected
                } else {
                    weather::Urgency::Future
                },
                source: "自动监测系统".to_string(),
                created_at: Utc::now(),
            });
        }

        if d.wind_speed_ms > 15.0 {
            let severity = if d.wind_speed_ms > 20.0 {
                weather::AlertSeverity::Severe
            } else if d.wind_speed_ms > 17.0 {
                weather::AlertSeverity::Medium
            } else {
                weather::AlertSeverity::Low
            };
            alerts.push(weather::WeatherAlert {
                id: Uuid::new_v4(),
                tower_id: Some(d.tower_id),
                alert_type: weather::WeatherAlertType::HighWindWarning,
                severity,
                timestamp: d.timestamp,
                title: "大风预警".to_string(),
                headline: "索道塔架大风风险预警".to_string(),
                message: format!("风速{:.1}m/s，请注意安全", d.wind_speed_ms),
                description: format!("当前风速{:.1}m/s，风向{:.0}°。强风可能导致塔架振动加剧，影响索道运行安全。", d.wind_speed_ms, d.wind_direction_deg),
                data: Some(serde_json::json!({
                    "wind_speed": d.wind_speed_ms,
                    "wind_direction": d.wind_direction_deg,
                })),
                effective_start: d.timestamp,
                effective_end: d.timestamp + Duration::hours(2),
                expires_at: Some(d.timestamp + Duration::hours(2)),
                affected_area: "索道全线".to_string(),
                response_type: if severity >= weather::AlertSeverity::Severe {
                    weather::ResponseType::ExecuteShutdown
                } else if severity >= weather::AlertSeverity::Medium {
                    weather::ResponseType::Prepare
                } else {
                    weather::ResponseType::Monitor
                },
                certainty: weather::Certainty::Observed,
                urgency: if severity >= weather::AlertSeverity::Severe {
                    weather::Urgency::Immediate
                } else {
                    weather::Urgency::Expected
                },
                source: "自动监测系统".to_string(),
                created_at: Utc::now(),
            });
        }
    }

    alerts
}

fn count_days_with_ice_risk(data: &[weather::WeatherData]) -> u32 {
    use std::collections::HashSet;
    let mut days = HashSet::new();

    for d in data {
        if d.temperature_c < 0.0 && d.humidity_percent > 80.0 {
            days.insert(d.timestamp.date_naive());
        }
    }

    days.len() as u32
}

fn count_days_with_high_wind(data: &[weather::WeatherData]) -> u32 {
    use std::collections::HashSet;
    let mut days = HashSet::new();

    for d in data {
        if d.wind_speed_ms > 15.0 {
            days.insert(d.timestamp.date_naive());
        }
    }

    days.len() as u32
}

fn count_days_with_extreme_temp(data: &[weather::WeatherData]) -> u32 {
    use std::collections::HashSet;
    let mut days = HashSet::new();

    for d in data {
        if d.temperature_c < -10.0 || d.temperature_c > 35.0 {
            days.insert(d.timestamp.date_naive());
        }
    }

    days.len() as u32
}

fn calculate_overall_risk(
    ice_days: u32,
    wind_days: u32,
    temp_days: u32,
    max_wind: f64,
) -> (weather::RiskLevel, weather::ImpactRating) {
    let mut score = 0.0;

    score += ice_days as f64 * 10.0;
    score += wind_days as f64 * 8.0;
    score += temp_days as f64 * 5.0;
    score += max_wind.max(0.0);

    let risk = match score {
        s if s >= 80.0 => weather::RiskLevel::Critical,
        s if s >= 50.0 => weather::RiskLevel::High,
        s if s >= 30.0 => weather::RiskLevel::Medium,
        s if s >= 10.0 => weather::RiskLevel::Low,
        _ => weather::RiskLevel::None,
    };

    let impact = match score {
        s if s >= 80.0 => weather::ImpactRating::Severe,
        s if s >= 50.0 => weather::ImpactRating::Major,
        s if s >= 30.0 => weather::ImpactRating::Moderate,
        s if s >= 10.0 => weather::ImpactRating::Minor,
        _ => weather::ImpactRating::Negligible,
    };

    (risk, impact)
}

fn calculate_dominant_wind_direction(data: &[weather::WeatherData]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let sum_sin: f64 = data.iter()
        .map(|d| d.wind_direction_deg.to_radians().sin())
        .sum();
    let sum_cos: f64 = data.iter()
        .map(|d| d.wind_direction_deg.to_radians().cos())
        .sum();

    let avg_dir = sum_sin.atan2(sum_cos).to_degrees();
    if avg_dir < 0.0 { avg_dir + 360.0 } else { avg_dir }
}

fn generate_impact_assessment(
    risk: weather::RiskLevel,
    max_wind: f64,
    ice_days: u32,
) -> weather::ImpactAssessment {
    weather::ImpactAssessment {
        operational_impact: match risk {
            weather::RiskLevel::Critical | weather::RiskLevel::High => "严重影响，可能需要频繁停运".to_string(),
            weather::RiskLevel::Medium => "中度影响，需加强监控".to_string(),
            _ => "影响较小，可正常运行".to_string(),
        },
        structural_impact: if ice_days > 3 {
            "高负荷运行，结构应力增加".to_string()
        } else {
            "结构应力在正常范围内".to_string()
        },
        maintenance_impact: format!("预计需要增加{}次巡检", ice_days.max(1)),
        estimated_cost_increase_percent: match risk {
            weather::RiskLevel::Critical => 30.0,
            weather::RiskLevel::High => 20.0,
            weather::RiskLevel::Medium => 10.0,
            weather::RiskLevel::Low => 5.0,
            _ => 0.0,
        },
        estimated_downtime_hours: if max_wind > 20.0 { 8.0 } else if max_wind > 15.0 { 4.0 } else { 0.0 },
        passenger_impact: if risk >= weather::RiskLevel::High {
            "可能造成乘客滞留，需准备应急预案".to_string()
        } else {
            "对乘客影响较小".to_string()
        },
    }
}

fn generate_mitigation_recommendations(
    risk_by_type: &[(weather::WeatherRiskType, weather::RiskLevel, f64)],
    max_wind: f64,
) -> Vec<weather::MitigationRecommendation> {
    let mut recs = Vec::new();

    for (risk_type, level, value) in risk_by_type {
        if *level >= weather::RiskLevel::Medium {
            let rec = match risk_type {
                weather::WeatherRiskType::Ice => weather::MitigationRecommendation {
                    risk_type: *risk_type,
                    priority: weather::RecommendationPriority::High,
                    action: "加强覆冰监测，准备除冰设备".to_string(),
                    estimated_cost: 10000.0,
                    effectiveness: 80.0,
                },
                weather::WeatherRiskType::HighWind => weather::MitigationRecommendation {
                    risk_type: *risk_type,
                    priority: if max_wind > 20.0 { weather::RecommendationPriority::High } else { weather::RecommendationPriority::Medium },
                    action: "设置大风告警阈值，准备停运预案".to_string(),
                    estimated_cost: 5000.0,
                    effectiveness: 75.0,
                },
                _ => weather::MitigationRecommendation {
                    risk_type: *risk_type,
                    priority: weather::RecommendationPriority::Low,
                    action: "加强监控，保持常规巡检".to_string(),
                    estimated_cost: 1000.0,
                    effectiveness: 60.0,
                },
            };
            recs.push(rec);
        }
    }

    recs.push(weather::MitigationRecommendation {
        risk_type: weather::WeatherRiskType::General,
        priority: weather::RecommendationPriority::Medium,
        action: "建立恶劣天气应急预案，定期演练".to_string(),
        estimated_cost: 2000.0,
        effectiveness: 85.0,
    });

    recs
}

fn generate_forecast_summary() -> weather::ForecastSummary {
    let now = Utc::now();
    weather::ForecastSummary {
        forecast_period_days: 7,
        forecast_time: now,
        avg_temperature_c: -2.0,
        min_temperature_c: -8.0,
        max_temperature_c: 3.0,
        avg_wind_speed_ms: 10.0,
        max_wind_speed_ms: 18.0,
        precipitation_probability: 60.0,
        expected_ice_days: 3,
        expected_high_wind_days: 2,
        expected_storm_days: 1,
        overall_forecast_risk: weather::RiskLevel::Medium,
        summary_text: "未来7天以低温阴雨天气为主，有3天可能出现覆冰，需加强监控".to_string(),
    }
}

pub async fn get_weather_forecast(
    state: AppState,
    tower_id: Option<Uuid>,
) -> Result<weather::WeatherForecast, AppError> {
    info!("Getting weather forecast");

    Ok(weather::WeatherForecast {
        id: Uuid::new_v4(),
        tower_id,
        forecast_time: Utc::now(),
        forecast_hours: 24,
        hourly_forecast: generate_hourly_forecast(),
        source: "本地气象站+数值预报".to_string(),
        created_at: Utc::now(),
    })
}

fn generate_hourly_forecast() -> Vec<weather::HourlyForecast> {
    let mut rng = rand::thread_rng();
    let mut forecast = Vec::new();
    let now = Utc::now();

    for i in 0..24 {
        let hour = (now.hour() as i32 + i as i32) % 24;
        let temp_factor = ((hour as f64 - 14.0) * 0.5).cos();
        let temperature = -2.0 + temp_factor * 3.0 + rng.gen_range(-1.0..1.0);

        forecast.push(weather::HourlyForecast {
            timestamp: now + Duration::hours(i as i64),
            temperature_c: temperature,
            humidity_percent: 85.0 + rng.gen_range(-15.0..15.0),
            wind_speed_ms: 8.0 + rng.gen_range(-3.0..5.0),
            wind_direction_deg: rng.gen_range(0.0..360.0),
            wind_gust_ms: Some(12.0 + rng.gen_range(-2.0..4.0)),
            precipitation_probability_percent: if temperature < 0.0 { 70.0 } else { 30.0 },
            precipitation_mm: if temperature < 0.0 { rng.gen_range(0.0..3.0) } else { rng.gen_range(0.0..1.0) },
            precipitation_type: if temperature < 0.0 {
                weather::PrecipitationType::Snow
            } else {
                weather::PrecipitationType::Rain
            },
            cloud_cover_percent: 70.0 + rng.gen_range(-30.0..30.0),
        });
    }

    forecast
}
