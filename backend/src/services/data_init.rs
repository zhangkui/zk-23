use crate::{AppState, AppError, models::*};
use chrono::{Utc, Duration};
use uuid::Uuid;
use tracing::info;

pub async fn init_sample_data(state: AppState) -> Result<(), AppError> {
    info!("Initializing sample data...");

    let cable_line_id = Uuid::new_v4();

    let user_id = Uuid::new_v4();
    let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST)?;
    let admin_user = user::User {
        id: user_id,
        username: "admin".to_string(),
        email: "admin@cableway.local".to_string(),
        full_name: "系统管理员".to_string(),
        role: user::UserRole::Admin,
        department: "技术部".to_string(),
        phone: Some("13800138000".to_string()),
        is_active: true,
        last_login: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    state.clickhouse_client.insert_user(&admin_user, &password_hash).await?;
    info!("Created default admin user: admin / admin123");

    let tower_names = vec!["一号塔架", "二号塔架", "三号塔架", "四号塔架", "五号塔架"];
    let base_lat = 30.12345;
    let base_lon = 103.45678;
    let base_elev = 1500.0;

    let mut tower_ids = Vec::new();

    for (i, name) in tower_names.iter().enumerate() {
        let tower_id = Uuid::new_v4();
        tower_ids.push(tower_id);

        let tower = tower::Tower {
            id: tower_id,
            name: name.to_string(),
            code: format!("TW-{:03}", i + 1),
            location: tower::Location {
                latitude: base_lat + (i as f64) * 0.001,
                longitude: base_lon + (i as f64) * 0.0015,
                elevation_meters: base_elev + (i as f64) * 50.0,
                description: Some(format!("{}位置描述", name)),
            },
            height_meters: 35.0 + (i as f64) * 2.5,
            construction_date: Some(Utc::now() - Duration::days(365 * 5)),
            status: tower::TowerStatus::Operational,
            cable_line_id,
            position_in_line: (i + 1) as u32,
            max_load_kg: 50000.0,
            last_inspection_date: Some(Utc::now() - Duration::days(30)),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            sensors: Vec::new(),
        };

        state.clickhouse_client.insert_tower(&tower).await?;
        info!("Created tower: {}", name);

        let sensor_types = vec![
            (sensor::SensorType::Vibration, "VIB-001", "振动传感器"),
            (sensor::SensorType::WindSpeed, "WIND-001", "风速传感器"),
            (sensor::SensorType::IceThickness, "ICE-001", "覆冰传感器"),
            (sensor::SensorType::Temperature, "TEMP-001", "温度传感器"),
            (sensor::SensorType::Humidity, "HUM-001", "湿度传感器"),
        ];

        for (sensor_type, device_prefix, model) in &sensor_types {
            let sensor_id = Uuid::new_v4();
            let sensor = sensor::Sensor {
                id: sensor_id,
                device_id: format!("{}-{}", device_prefix, i + 1),
                sensor_type: *sensor_type,
                model: model.to_string(),
                manufacturer: "国产".to_string(),
                status: sensor::SensorStatus::Active,
                calibration_date: Some(Utc::now() - Duration::days(30)),
                last_calibration_date: Some(Utc::now() - Duration::days(30)),
                calibration_interval_days: 90,
                sampling_rate_hz: match sensor_type {
                    sensor::SensorType::Vibration => 100.0,
                    sensor::SensorType::WindSpeed => 10.0,
                    _ => 1.0,
                },
                measurement_range_min: match sensor_type {
                    sensor::SensorType::Vibration => 0.0,
                    sensor::SensorType::WindSpeed => 0.0,
                    sensor::SensorType::IceThickness => 0.0,
                    sensor::SensorType::Temperature => -40.0,
                    sensor::SensorType::Humidity => 0.0,
                    _ => 0.0,
                },
                measurement_range_max: match sensor_type {
                    sensor::SensorType::Vibration => 100.0,
                    sensor::SensorType::WindSpeed => 60.0,
                    sensor::SensorType::IceThickness => 100.0,
                    sensor::SensorType::Temperature => 60.0,
                    sensor::SensorType::Humidity => 100.0,
                    _ => 100.0,
                },
                accuracy: match sensor_type {
                    sensor::SensorType::Vibration => 0.1,
                    sensor::SensorType::WindSpeed => 0.1,
                    sensor::SensorType::IceThickness => 0.1,
                    sensor::SensorType::Temperature => 0.1,
                    sensor::SensorType::Humidity => 1.0,
                    _ => 0.1,
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            state.clickhouse_client.insert_sensor(&sensor).await?;
        }

        let camera_id = Uuid::new_v4();
        let camera = video::Camera {
            id: camera_id,
            tower_id,
            device_id: format!("CAM-{:03}", i + 1),
            name: format!("{}监控摄像头", name),
            location: format!("{}顶部", name),
            mount_position: "塔顶".to_string(),
            camera_type: video::CameraType::PTZ,
            status: video::CameraStatus::Online,
            rtsp_url: Some(format!("rtsp://192.168.1.100:554/stream{}", i + 1)),
            http_url: Some(format!("http://192.168.1.100:8080/stream{}", i + 1)),
            resolution: "1920x1080".to_string(),
            fps: 25,
            has_ai_analysis: true,
            ai_model_version: Some("v2.1.0".to_string()),
            last_online: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        info!("Created sensors and camera for tower: {}", name);
    }

    info!("Sample data initialization complete");
    Ok(())
}
