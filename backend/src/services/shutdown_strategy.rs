use crate::{AppState, AppError, models::*, mq};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};

pub async fn evaluate_shutdown_strategy(
    state: AppState,
    tower_id: Uuid,
    alert_id: Uuid,
) -> Result<shutdown_strategy::ShutdownStrategy, AppError> {
    info!("Evaluating shutdown strategy for tower {}, alert {}", tower_id, alert_id);

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

    let max_ice = ice_data.iter().map(|d| d.ice_thickness_mm).fold(0.0, f64::max);
    let max_wind = wind_data.iter().map(|d| d.velocity_mm_s).fold(0.0, f64::max);
    let max_vibration = vibration_data.iter().map(|d| d.velocity_mm_s).fold(0.0, f64::max);

    let (shutdown_type, urgency, risk_level) = determine_shutdown_type(max_ice, max_wind, max_vibration);

    let trigger_conditions = generate_trigger_conditions(max_ice, max_wind, max_vibration);

    let action_steps = generate_action_steps(shutdown_type, urgency);

    let restart_plan = generate_restart_plan(max_ice, max_wind);

    let strategy = shutdown_strategy::ShutdownStrategy {
        id: Uuid::new_v4(),
        tower_id: Some(tower_id),
        cable_line_id: None,
        alert_id: Some(alert_id),
        shutdown_type,
        strategy_name: format!("{:?}停运策略", shutdown_type),
        description: format!("基于覆冰{:.1}mm、风速{:.1}m/s、振动{:.1}mm/s的停运评估",
            max_ice, max_wind, max_vibration),
        risk_level,
        urgency,
        trigger_conditions,
        action_steps,
        restart_plan,
        estimated_duration_minutes: if urgency == shutdown_strategy::UrgencyLevel::Immediate {
            Some(120)
        } else if urgency == shutdown_strategy::UrgencyLevel::High {
            Some(60)
        } else {
            Some(30)
        },
        estimated_restart_time: restart_plan.estimated_restart_time,
        status: shutdown_strategy::StrategyStatus::Pending,
        approved_by: None,
        approved_at: None,
        executed_at: None,
        created_by: None,
        notes: Some("自动生成的停运策略，需人工确认后执行".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        version: 1,
    };

    if urgency == shutdown_strategy::UrgencyLevel::Immediate {
        warn!("Immediate shutdown recommended for tower {}: ice={:.1}mm, wind={:.1}m/s",
            tower_id, max_ice, max_wind);
    }

    Ok(strategy)
}

fn determine_shutdown_type(
    ice_thickness: f64,
    wind_speed: f64,
    vibration: f64,
) -> (shutdown_strategy::ShutdownType, shutdown_strategy::UrgencyLevel, shutdown_strategy::RiskLevel) {
    let mut score = 0.0;

    if ice_thickness > 15.0 { score += 40.0; }
    else if ice_thickness > 10.0 { score += 30.0; }
    else if ice_thickness > 5.0 { score += 15.0; }

    if wind_speed > 25.0 { score += 35.0; }
    else if wind_speed > 20.0 { score += 25.0; }
    else if wind_speed > 15.0 { score += 15.0; }

    if vibration > 5.0 { score += 25.0; }
    else if vibration > 3.0 { score += 15.0; }
    else if vibration > 2.0 { score += 8.0; }

    let (shutdown_type, urgency, risk) = match score {
        s if s >= 80.0 => (
            shutdown_strategy::ShutdownType::Emergency,
            shutdown_strategy::UrgencyLevel::Immediate,
            shutdown_strategy::RiskLevel::Critical,
        ),
        s if s >= 60.0 => (
            shutdown_strategy::ShutdownType::Preventive,
            shutdown_strategy::UrgencyLevel::High,
            shutdown_strategy::RiskLevel::High,
        ),
        s if s >= 40.0 => (
            shutdown_strategy::ShutdownType::Preventive,
            shutdown_strategy::UrgencyLevel::Medium,
            shutdown_strategy::RiskLevel::Medium,
        ),
        s if s >= 20.0 => (
            shutdown_strategy::ShutdownType::Conditional,
            shutdown_strategy::UrgencyLevel::Low,
            shutdown_strategy::RiskLevel::Low,
        ),
        _ => (
            shutdown_strategy::ShutdownType::None,
            shutdown_strategy::UrgencyLevel::None,
            shutdown_strategy::RiskLevel::None,
        ),
    };

    (shutdown_type, urgency, risk)
}

fn generate_trigger_conditions(
    ice_thickness: f64,
    wind_speed: f64,
    vibration: f64,
) -> Vec<shutdown_strategy::TriggerCondition> {
    let mut conditions = Vec::new();

    if ice_thickness > 5.0 {
        conditions.push(shutdown_strategy::TriggerCondition {
            condition_type: shutdown_strategy::ConditionType::IceThickness,
            operator: ">=".to_string(),
            threshold: 10.0,
            current_value: ice_thickness,
            unit: "mm".to_string(),
            triggered: ice_thickness >= 10.0,
            weight: 0.4,
        });
    }

    if wind_speed > 10.0 {
        conditions.push(shutdown_strategy::TriggerCondition {
            condition_type: shutdown_strategy::ConditionType::WindSpeed,
            operator: ">=".to_string(),
            threshold: 20.0,
            current_value: wind_speed,
            unit: "m/s".to_string(),
            triggered: wind_speed >= 20.0,
            weight: 0.35,
        });
    }

    if vibration > 2.0 {
        conditions.push(shutdown_strategy::TriggerCondition {
            condition_type: shutdown_strategy::ConditionType::Vibration,
            operator: ">=".to_string(),
            threshold: 3.0,
            current_value: vibration,
            unit: "mm/s".to_string(),
            triggered: vibration >= 3.0,
            weight: 0.25,
        });
    }

    conditions
}

fn generate_action_steps(
    shutdown_type: shutdown_strategy::ShutdownType,
    urgency: shutdown_strategy::UrgencyLevel,
) -> Vec<shutdown_strategy::ActionStep> {
    let mut steps = Vec::new();

    match urgency {
        shutdown_strategy::UrgencyLevel::Immediate => {
            steps.push(shutdown_strategy::ActionStep {
                order: 1,
                title: "立即停止索道运行".to_string(),
                description: "按下紧急停止按钮，停止所有轿厢运行".to_string(),
                responsible_role: "操作员".to_string(),
                estimated_duration_minutes: 1,
                critical: true,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
            steps.push(shutdown_strategy::ActionStep {
                order: 2,
                title: "疏散索道乘客".to_string(),
                description: "通过广播安抚乘客，启动应急疏散程序".to_string(),
                responsible_role: "安全员".to_string(),
                estimated_duration_minutes: 30,
                critical: true,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
            steps.push(shutdown_strategy::ActionStep {
                order: 3,
                title: "关闭电源".to_string(),
                description: "切断主电源，防止意外启动".to_string(),
                responsible_role: "电工".to_string(),
                estimated_duration_minutes: 5,
                critical: true,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
        }
        shutdown_strategy::UrgencyLevel::High => {
            steps.push(shutdown_strategy::ActionStep {
                order: 1,
                title: "做好停运准备".to_string(),
                description: "通知所有岗位人员准备停运".to_string(),
                responsible_role: "值班主任".to_string(),
                estimated_duration_minutes: 5,
                critical: false,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
            steps.push(shutdown_strategy::ActionStep {
                order: 2,
                title: "将轿厢运行至就近车站".to_string(),
                description: "低速运行，确保所有乘客安全下车".to_string(),
                responsible_role: "操作员".to_string(),
                estimated_duration_minutes: 15,
                critical: true,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
            steps.push(shutdown_strategy::ActionStep {
                order: 3,
                title: "停止索道运行".to_string(),
                description: "确认所有乘客下车后，停止运行".to_string(),
                responsible_role: "操作员".to_string(),
                estimated_duration_minutes: 2,
                critical: true,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
        }
        shutdown_strategy::UrgencyLevel::Medium => {
            steps.push(shutdown_strategy::ActionStep {
                order: 1,
                title: "加强监控".to_string(),
                description: "密切关注各参数变化，每5分钟报告一次".to_string(),
                responsible_role: "监控员".to_string(),
                estimated_duration_minutes: 0,
                critical: false,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
            steps.push(shutdown_strategy::ActionStep {
                order: 2,
                title: "准备停运预案".to_string(),
                description: "制定详细的停运计划，等待进一步指令".to_string(),
                responsible_role: "值班主任".to_string(),
                estimated_duration_minutes: 10,
                critical: false,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
        }
        _ => {
            steps.push(shutdown_strategy::ActionStep {
                order: 1,
                title: "正常运行".to_string(),
                description: "继续正常运行，保持常规监控".to_string(),
                responsible_role: "全体".to_string(),
                estimated_duration_minutes: 0,
                critical: false,
                executed: false,
                executed_at: None,
                executed_by: None,
            });
        }
    }

    steps
}

fn generate_restart_plan(
    ice_thickness: f64,
    wind_speed: f64,
) -> shutdown_strategy::RestartPlan {
    let (estimated_hours, condition) = if ice_thickness > 10.0 {
        (48, "覆冰厚度降低至5mm以下，风速降至15m/s以下")
    } else if ice_thickness > 5.0 {
        (24, "覆冰厚度降低至3mm以下，风速降至15m/s以下")
    } else if wind_speed > 20.0 {
        (12, "风速降至15m/s以下并保持2小时以上")
    } else {
        (6, "所有参数恢复至正常范围并保持1小时以上")
    };

    shutdown_strategy::RestartPlan {
        conditions: vec![
            condition.to_string(),
            "完成塔架及线路安全巡检".to_string(),
            "确认无结构损坏或异常".to_string(),
            "完成空载试运行".to_string(),
            "获得运营主管批准".to_string(),
        ],
        estimated_restart_time: Some(Utc::now() + Duration::hours(estimated_hours)),
        pre_restart_checks: vec![
            "检查所有塔架结构完整性".to_string(),
            "检查线路张力是否正常".to_string(),
            "检查所有传感器是否正常工作".to_string(),
            "检查通信系统是否正常".to_string(),
            "清理站台和相关设施".to_string(),
        ],
        post_restart_procedures: vec![
            "空载试运行30分钟".to_string(),
            "低速运行15分钟".to_string(),
            "逐步恢复正常运行速度".to_string(),
            "运行首趟载人列车后进行安全检查".to_string(),
        ],
        responsible_roles: vec![
            "值班主任".to_string(),
            "运维主管".to_string(),
            "安全主管".to_string(),
        ],
    }
}

pub async fn execute_shutdown(
    state: AppState,
    strategy_id: Uuid,
    executed_by: Uuid,
    notes: Option<String>,
) -> Result<shutdown_strategy::ShutdownLog, AppError> {
    info!("Executing shutdown strategy {} by user {}", strategy_id, executed_by);

    let log = shutdown_strategy::ShutdownLog {
        id: Uuid::new_v4(),
        strategy_id: Some(strategy_id),
        tower_id: None,
        cable_line_id: None,
        shutdown_type: shutdown_strategy::ShutdownType::Preventive,
        reason: Some("根据系统推荐策略执行停运".to_string()),
        executed_by,
        started_at: Utc::now(),
        completed_at: None,
        status: shutdown_strategy::ShutdownStatus::InProgress,
        passengers_evacuated: 0,
        cars_returned: 0,
        total_cars: 50,
        incidents_reported: vec![],
        notes,
        created_at: Utc::now(),
    };

    state.clickhouse_client.insert_shutdown_log(&log).await?;

    mq::publisher::publish_shutdown_executed(&state, &log).await?;

    Ok(log)
}

pub async fn get_active_strategies(
    state: AppState,
    tower_id: Option<Uuid>,
) -> Result<Vec<shutdown_strategy::ShutdownStrategy>, AppError> {
    info!("Getting active shutdown strategies");
    Ok(vec![])
}

pub async fn evaluate_strategies(
    state: &AppState,
    tower_id: Option<Uuid>,
) -> Result<(), AppError> {
    if let Some(tower_id) = tower_id {
        info!("Evaluating shutdown strategies for tower {}", tower_id);
    } else {
        info!("Evaluating shutdown strategies for all towers");
    }
    Ok(())
}
