use crate::{AppState, AppError, models::*, mq};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::{info, warn};

pub async fn request_auto_verification(
    state: &AppState,
    alert_msg: &alert::AlertMessage,
) -> Result<video::VideoVerificationResult, AppError> {
    let tower_id = alert_msg.tower_id.ok_or_else(|| AppError::Validation("Alert missing tower_id".to_string()))?;
    let requested_by = Uuid::nil();
    let priority = match alert_msg.severity {
        alert::AlertSeverity::Critical => video::VerificationPriority::Critical,
        alert::AlertSeverity::High => video::VerificationPriority::High,
        alert::AlertSeverity::Medium => video::VerificationPriority::Medium,
        _ => video::VerificationPriority::Low,
    };

    request_video_verification(
        state.clone(),
        tower_id,
        alert_msg.id,
        requested_by,
        priority,
    ).await
}

pub async fn request_video_verification(
    state: AppState,
    tower_id: Uuid,
    alert_id: Uuid,
    requested_by: Uuid,
    priority: video::VerificationPriority,
) -> Result<video::VideoVerificationResult, AppError> {
    info!("Requesting video verification for tower {}, alert {}", tower_id, alert_id);

    let request_id = Uuid::new_v4();
    let now = Utc::now();

    let request = video::VideoVerificationRequest {
        id: request_id,
        tower_id,
        alert_id: Some(alert_id),
        requested_by,
        requested_at: now,
        priority,
        status: video::VerificationStatus::Pending,
        analysis_type: vec![
            video::AIAnalysisType::IceDetection,
            video::AIAnalysisType::StructureInspection,
        ],
        expires_at: Some(now + Duration::minutes(30)),
    };

    mq::publisher::publish_video_verification_requested(&state, &request).await?;

    let simulated_result = simulate_ai_analysis(tower_id, request_id, alert_id, requested_by).await?;

    mq::publisher::publish_video_verification_completed(&state, &simulated_result).await?;

    Ok(simulated_result)
}

async fn simulate_ai_analysis(
    tower_id: Uuid,
    request_id: Uuid,
    alert_id: Uuid,
    requested_by: Uuid,
) -> Result<video::VideoVerificationResult, AppError> {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let ice_confidence = rng.gen_range(70.0..95.0);
    let structure_confidence = rng.gen_range(80.0..98.0);
    let ice_detected = ice_confidence > 75.0;
    let ice_thickness = if ice_detected {
        Some(rng.gen_range(3.0..12.0))
    } else {
        None
    };

    let ai_results = vec![
        video::AIAnalysisResult {
            analysis_type: video::AIAnalysisType::IceDetection,
            confidence: ice_confidence,
            detected: ice_detected,
            description: if ice_detected {
                format!("检测到覆冰，厚度约{:.1}mm", ice_thickness.unwrap_or(0.0))
            } else {
                "未检测到明显覆冰".to_string()
            },
            bounding_boxes: if ice_detected {
                Some(vec![
                    video::BoundingBox {
                        x: 100, y: 150, width: 200, height: 150, label: "ice".to_string(), confidence: ice_confidence,
                    }
                ])
            } else {
                None
            },
            metadata: serde_json::json!({
                "ice_thickness_mm": ice_thickness,
                "ice_type": if ice_thickness.unwrap_or(0.0) > 8.0 { "glaze" } else { "rime" },
            }),
        },
        video::AIAnalysisResult {
            analysis_type: video::AIAnalysisType::StructureInspection,
            confidence: structure_confidence,
            detected: false,
            description: "塔架结构正常，未发现明显异常".to_string(),
            bounding_boxes: None,
            metadata: serde_json::json!({
                "cable_tension": "normal",
                "tower_alignment": "normal",
            }),
        },
    ];

    let overall_confidence = ai_results.iter().map(|r| r.confidence).sum::<f64>() / ai_results.len() as f64;
    let any_detection = ai_results.iter().any(|r| r.detected);

    let result = video::VideoVerificationResult {
        id: Uuid::new_v4(),
        request_id,
        tower_id,
        alert_id: Some(alert_id),
        camera_id: Some(Uuid::new_v4()),
        verification_started_at: Utc::now() - Duration::seconds(2),
        verification_completed_at: Utc::now(),
        status: video::VerificationStatus::Completed,
        ai_results,
        manual_verification_required: overall_confidence < 85.0 || any_detection,
        verified_by: None,
        verified_at: None,
        notes: None,
        overall_confidence,
        ice_thickness_estimated_mm: ice_thickness,
        ice_confirmed: ice_detected,
        video_clip_url: Some(format!("/api/v1/video/clips/{}.mp4", request_id)),
        snapshot_url: Some(format!("/api/v1/video/snapshots/{}.jpg", request_id)),
        created_at: Utc::now(),
    };

    info!("Video verification completed for request {}: confidence={:.1}%, ice_detected={}",
        request_id, overall_confidence, ice_detected);

    Ok(result)
}

pub async fn start_live_stream(
    state: AppState,
    tower_id: Uuid,
    user_id: Uuid,
) -> Result<video::LiveStreamSession, AppError> {
    info!("Starting live stream for tower {} by user {}", tower_id, user_id);

    let towers = state.clickhouse_client.list_towers(&tower::TowerListQuery {
        cable_line_id: None,
        status: None,
        page: Some(0),
        page_size: Some(100),
    }).await?;

    let tower = towers.iter().find(|t| t.id == tower_id)
        .ok_or_else(|| AppError::NotFound(format!("Tower {} not found", tower_id)))?;

    let session = video::LiveStreamSession {
        session_id: Uuid::new_v4(),
        tower_id,
        camera_id: Uuid::new_v4(),
        user_id,
        started_at: Utc::now(),
        expires_at: Utc::now() + Duration::minutes(30),
        stream_url: format!("ws://192.168.1.100:8080/stream/{}", tower.code),
        stream_type: video::StreamType::WebRTC,
        quality: "720p".to_string(),
        is_recording: false,
    };

    Ok(session)
}

pub async fn stop_live_stream(
    state: AppState,
    session_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    info!("Stopping live stream session {} by user {}", session_id, user_id);
    Ok(())
}

pub async fn manual_verify(
    state: AppState,
    result_id: Uuid,
    user_id: Uuid,
    ice_confirmed: bool,
    notes: String,
) -> Result<video::VideoVerificationResult, AppError> {
    info!("Manual verification for result {} by user {}", result_id, user_id);

    Ok(video::VideoVerificationResult {
        id: result_id,
        request_id: Uuid::new_v4(),
        tower_id: Uuid::new_v4(),
        alert_id: None,
        camera_id: None,
        verification_started_at: Utc::now() - Duration::minutes(5),
        verification_completed_at: Utc::now(),
        status: video::VerificationStatus::Completed,
        ai_results: vec![],
        manual_verification_required: false,
        verified_by: Some(user_id),
        verified_at: Some(Utc::now()),
        notes: Some(notes),
        overall_confidence: 100.0,
        ice_thickness_estimated_mm: if ice_confirmed { Some(10.0) } else { None },
        ice_confirmed,
        video_clip_url: None,
        snapshot_url: None,
        created_at: Utc::now(),
    })
}

pub async fn get_camera_list(
    state: AppState,
    tower_id: Option<Uuid>,
) -> Result<Vec<video::Camera>, AppError> {
    let towers = state.clickhouse_client.list_towers(&tower::TowerListQuery {
        cable_line_id: None,
        status: None,
        page: Some(0),
        page_size: Some(100),
    }).await?;

    let mut cameras = Vec::new();

    for tower in &towers {
        if let Some(tid) = tower_id {
            if tid != tower.id {
                continue;
            }
        }

        cameras.push(video::Camera {
            id: Uuid::new_v4(),
            tower_id: tower.id,
            device_id: format!("CAM-{:03}", tower.position_in_line),
            name: format!("{}监控摄像头", tower.name),
            location: format!("{}顶部", tower.name),
            mount_position: "塔顶".to_string(),
            camera_type: video::CameraType::PTZ,
            status: video::CameraStatus::Online,
            rtsp_url: Some(format!("rtsp://192.168.1.100:554/stream{}", tower.position_in_line)),
            http_url: Some(format!("http://192.168.1.100:8080/stream{}", tower.position_in_line)),
            resolution: "1920x1080".to_string(),
            fps: 25,
            has_ai_analysis: true,
            ai_model_version: Some("v2.1.0".to_string()),
            last_online: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
    }

    Ok(cameras)
}
