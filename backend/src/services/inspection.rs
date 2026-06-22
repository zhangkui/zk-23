use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info};

pub async fn create_inspection_record(
    state: AppState,
    record: &inspection::InspectionRecord,
) -> Result<Uuid, AppError> {
    info!("Creating inspection record for tower {}", record.tower_id);

    state.clickhouse_client.insert_inspection(record).await?;

    Ok(record.id)
}

pub async fn get_inspection_records(
    state: AppState,
    tower_id: Option<Uuid>,
    inspection_type: Option<inspection::InspectionType>,
    start_time: Option<chrono::DateTime<Utc>>,
    end_time: Option<chrono::DateTime<Utc>>,
    page: u32,
    page_size: u32,
) -> Result<Vec<inspection::InspectionRecord>, AppError> {
    info!("Getting inspection records");

    let all_records = generate_sample_inspections();

    let filtered: Vec<_> = all_records.into_iter()
        .filter(|r| tower_id.map_or(true, |id| r.tower_id == id))
        .filter(|r| inspection_type.as_ref().map_or(true, |t| r.inspection_type == *t))
        .filter(|r| start_time.map_or(true, |st| r.inspection_date >= st))
        .filter(|r| end_time.map_or(true, |et| r.inspection_date <= et))
        .skip((page * page_size) as usize)
        .take(page_size as usize)
        .collect();

    Ok(filtered)
}

pub async fn get_inspection_by_id(
    state: AppState,
    id: Uuid,
) -> Result<inspection::InspectionRecord, AppError> {
    let all_records = generate_sample_inspections();

    all_records.into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Inspection record {} not found", id)))
}

pub async fn create_maintenance_task(
    state: AppState,
    task: &inspection::MaintenanceTask,
) -> Result<Uuid, AppError> {
    info!("Creating maintenance task: {}", task.title);
    Ok(task.id)
}

pub async fn get_maintenance_tasks(
    state: AppState,
    tower_id: Option<Uuid>,
    status: Option<inspection::MaintenanceStatus>,
    priority: Option<inspection::MaintenancePriority>,
) -> Result<Vec<inspection::MaintenanceTask>, AppError> {
    info!("Getting maintenance tasks");

    let tasks = generate_sample_maintenance_tasks();

    let filtered: Vec<_> = tasks.into_iter()
        .filter(|t| tower_id.map_or(true, |id| t.tower_id == id))
        .filter(|t| status.as_ref().map_or(true, |s| t.status == *s))
        .filter(|t| priority.as_ref().map_or(true, |p| t.priority == *p))
        .collect();

    Ok(filtered)
}

pub async fn update_maintenance_task_status(
    state: AppState,
    task_id: Uuid,
    status: inspection::MaintenanceStatus,
    updated_by: Uuid,
    notes: Option<String>,
) -> Result<(), AppError> {
    info!("Updating maintenance task {} status to {:?}", task_id, status);
    Ok(())
}

pub async fn generate_inspection_report(
    state: AppState,
    tower_id: Uuid,
    start_time: chrono::DateTime<Utc>,
    end_time: chrono::DateTime<Utc>,
) -> Result<inspection::InspectionReport, AppError> {
    info!("Generating inspection report for tower {} from {:?} to {:?}",
        tower_id, start_time, end_time);

    let inspections = get_inspection_records(
        state.clone(),
        Some(tower_id),
        None,
        Some(start_time),
        Some(end_time),
        0,
        1000,
    ).await?;

    let tasks = get_maintenance_tasks(
        state.clone(),
        Some(tower_id),
        None,
        None,
    ).await?;

    let completed_tasks = tasks.iter()
        .filter(|t| t.status == inspection::MaintenanceStatus::Completed)
        .count();

    let pending_tasks = tasks.iter()
        .filter(|t| t.status == inspection::MaintenanceStatus::Pending || t.status == inspection::MaintenanceStatus::InProgress)
        .count();

    let high_priority_tasks = tasks.iter()
        .filter(|t| t.priority == inspection::MaintenancePriority::High || t.priority == inspection::MaintenancePriority::Critical)
        .count();

    let total_findings = inspections.iter()
        .map(|i| i.findings.len())
        .sum::<usize>();

    let critical_findings = inspections.iter()
        .map(|i| i.findings.iter()
            .filter(|f| f.severity == inspection::FindingSeverity::Critical)
            .count())
        .sum::<usize>();

    Ok(inspection::InspectionReport {
        id: Uuid::new_v4(),
        tower_id,
        report_period_start: start_time,
        report_period_end: end_time,
        generated_at: Utc::now(),
        total_inspections: inspections.len() as u32,
        total_findings: total_findings as u32,
        critical_findings: critical_findings as u32,
        resolved_findings: (total_findings - critical_findings) as u32,
        maintenance_tasks_created: tasks.len() as u32,
        maintenance_tasks_completed: completed_tasks as u32,
        maintenance_tasks_pending: pending_tasks as u32,
        high_priority_tasks: high_priority_tasks as u32,
        average_inspection_score: if !inspections.is_empty() {
            inspections.iter().map(|i| i.overall_rating.unwrap_or(80.0)).sum::<f64>() / inspections.len() as f64
        } else {
            0.0
        },
        recommendations: generate_recommendations(critical_findings, high_priority_tasks),
        next_inspection_date: Utc::now() + Duration::days(30),
        prepared_by: Some("系统自动生成".to_string()),
    })
}

fn generate_recommendations(critical_findings: usize, high_priority_tasks: usize) -> Vec<String> {
    let mut recs = Vec::new();

    if critical_findings > 0 {
        recs.push(format!("立即处理{}项严重缺陷", critical_findings));
    }

    if high_priority_tasks > 0 {
        recs.push(format!("优先处理{}项高优先级维护任务", high_priority_tasks));
    }

    recs.push("加强冬季覆冰季节的巡检频率".to_string());
    recs.push("定期校准传感器，确保数据准确性".to_string());
    recs.push("制定应急预案，演练应急响应流程".to_string());

    recs
}

fn generate_sample_inspections() -> Vec<inspection::InspectionRecord> {
    let mut inspections = Vec::new();
    let now = Utc::now();

    for i in 0..5 {
        let inspection_date = now - Duration::days((i * 7) as i64);
        inspections.push(inspection::InspectionRecord {
            id: Uuid::new_v4(),
            tower_id: Uuid::new_v4(),
            inspection_type: if i % 2 == 0 {
                inspection::InspectionType::Routine
            } else {
                inspection::InspectionType::Special
            },
            inspection_date,
            inspector_name: "张三".to_string(),
            inspector_id: Uuid::new_v4(),
            weather_conditions: "晴".to_string(),
            temperature_c: Some(5.0 + i as f64),
            wind_speed_ms: Some(3.0 + i as f64),
            check_items: generate_check_items(),
            findings: generate_findings(i),
            overall_rating: Some(80.0 + (5 - i) as f64 * 4.0),
            notes: Some(format!("第{}次常规巡检记录", i + 1)),
            photos: vec![],
            next_inspection_date: Some(inspection_date + Duration::days(30)),
            created_at: inspection_date,
            updated_at: inspection_date,
        });
    }

    inspections
}

fn generate_check_items() -> Vec<inspection::InspectionCheckItem> {
    vec![
        inspection::InspectionCheckItem {
            item_code: "FOUNDATION-001".to_string(),
            item_name: "基础检查".to_string(),
            category: inspection::CheckCategory::Foundation,
            result: inspection::CheckResult::Pass,
            notes: Some("基础牢固，无沉降").to_string(),
            photos: vec![],
        },
        inspection::InspectionCheckItem {
            item_code: "STRUCTURE-001".to_string(),
            item_name: "钢结构检查".to_string(),
            category: inspection::CheckCategory::Structure,
            result: inspection::CheckResult::Pass,
            notes: Some("钢结构完好，无锈蚀".to_string()),
            photos: vec![],
        },
        inspection::InspectionCheckItem {
            item_code: "CABLE-001".to_string(),
            item_name: "钢索检查".to_string(),
            category: inspection::CheckCategory::Cable,
            result: inspection::CheckResult::Pass,
            notes: Some("钢索张力正常，无断丝".to_string()),
            photos: vec![],
        },
        inspection::InspectionCheckItem {
            item_code: "SENSOR-001".to_string(),
            item_name: "传感器检查".to_string(),
            category: inspection::CheckCategory::Sensor,
            result: inspection::CheckResult::Pass,
            notes: Some("所有传感器工作正常".to_string()),
            photos: vec![],
        },
    ]
}

fn generate_findings(seed: i32) -> Vec<inspection::InspectionFinding> {
    let mut findings = Vec::new();

    if seed % 3 == 0 {
        findings.push(inspection::InspectionFinding {
            id: Uuid::new_v4(),
            finding_type: inspection::FindingType::Corrosion,
            severity: inspection::FindingSeverity::Low,
            location: "三号横梁".to_string(),
            description: "发现轻微锈蚀".to_string(),
            recommendation: "建议下次巡检时除锈涂漆".to_string(),
            photos: vec![],
            requires_maintenance: true,
            resolved: seed >= 2,
            resolved_at: if seed >= 2 { Some(Utc::now()) } else { None },
            maintenance_task_id: None,
            created_at: Utc::now(),
        });
    }

    if seed % 4 == 0 {
        findings.push(inspection::InspectionFinding {
            id: Uuid::new_v4(),
            finding_type: inspection::FindingType::LooseConnection,
            severity: inspection::FindingSeverity::Medium,
            location: "传感器安装架".to_string(),
            description: "传感器安装螺栓松动".to_string(),
            recommendation: "立即紧固螺栓，重新校准传感器".to_string(),
            photos: vec![],
            requires_maintenance: true,
            resolved: true,
            resolved_at: Some(Utc::now()),
            maintenance_task_id: Some(Uuid::new_v4()),
            created_at: Utc::now(),
        });
    }

    findings
}

fn generate_sample_maintenance_tasks() -> Vec<inspection::MaintenanceTask> {
    let mut tasks = Vec::new();
    let now = Utc::now();

    for i in 0..3 {
        tasks.push(inspection::MaintenanceTask {
            id: Uuid::new_v4(),
            tower_id: Uuid::new_v4(),
            inspection_id: Some(Uuid::new_v4()),
            finding_id: Some(Uuid::new_v4()),
            title: format!("维护任务 #{}", i + 1),
            description: "处理巡检发现的问题".to_string(),
            task_type: inspection::MaintenanceType::Repair,
            priority: match i {
                0 => inspection::MaintenancePriority::Critical,
                1 => inspection::MaintenancePriority::High,
                _ => inspection::MaintenancePriority::Medium,
            },
            status: match i {
                0 => inspection::MaintenanceStatus::Pending,
                1 => inspection::MaintenanceStatus::InProgress,
                _ => inspection::MaintenanceStatus::Completed,
            },
            assigned_to: Some("李四".to_string()),
            assigned_user_id: Some(Uuid::new_v4()),
            due_date: now + Duration::days((i + 1) * 7),
            estimated_hours: Some(4.0),
            actual_hours: if i == 2 { Some(3.5) } else { None },
            completed_at: if i == 2 { Some(now) } else { None },
            materials_used: vec![],
            labor_cost: if i == 2 { Some(500.0) } else { None },
            material_cost: if i == 2 { Some(200.0) } else { None },
            notes: None,
            created_at: now,
            updated_at: now,
        });
    }

    tasks
}
