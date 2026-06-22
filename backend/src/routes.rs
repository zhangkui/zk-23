use crate::{AppState, handlers, websocket};
use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware,
};

pub fn create_routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/api/auth/login", post(handlers::auth::login))
        .route("/health", get(handlers::health::health_check))
        .route("/health/ready", get(handlers::health::readiness_check))
        .route("/health/alive", get(handlers::health::liveness_check))
        .route("/ws", get(websocket::ws_handler))
        .with_state(state.clone());

    let protected_routes = Router::new()
        .route("/api/auth/me", get(handlers::auth::get_current_user))
        .route("/api/auth/change-password", post(handlers::auth::change_password))

        .route("/api/towers", get(handlers::tower::list_towers))
        .route("/api/towers", post(handlers::tower::create_tower))
        .route("/api/towers/:id", get(handlers::tower::get_tower))
        .route("/api/towers/:id", put(handlers::tower::update_tower))
        .route("/api/towers/:id", delete(handlers::tower::delete_tower))
        .route("/api/towers/:id/status", get(handlers::tower::get_tower_status))
        .route("/api/towers/status/all", get(handlers::tower::get_all_towers_status))

        .route("/api/towers/:tower_id/sensors", post(handlers::sensor::create_sensor))
        .route("/api/sensors", get(handlers::sensor::list_sensors))
        .route("/api/sensors/:id", get(handlers::sensor::get_sensor))
        .route("/api/sensors/:id/status", put(handlers::sensor::update_sensor_status))
        .route("/api/sensors/:id/calibrate", post(handlers::sensor::calibrate_sensor))

        .route("/api/data/:tower_id", get(handlers::data::get_sensor_data))
        .route("/api/data/:tower_id/vibration", get(handlers::data::get_vibration_data))
        .route("/api/data/:tower_id/wind", get(handlers::data::get_wind_data))
        .route("/api/data/:tower_id/ice", get(handlers::data::get_ice_data))
        .route("/api/data/:tower_id/weather", get(handlers::weather::get_weather_data))
        .route("/api/data/:tower_id/:data_type", post(handlers::data::ingest_sensor_data))

        .route("/api/alerts", get(handlers::alert::get_alerts))
        .route("/api/alerts/summary", get(handlers::alert::get_alert_summary))
        .route("/api/alerts/:id", get(handlers::alert::get_alert))
        .route("/api/alerts/:id/acknowledge", post(handlers::alert::acknowledge_alert))
        .route("/api/alerts/:id/resolve", post(handlers::alert::resolve_alert))

        .route("/api/ice-risk/:tower_id/analyze", post(handlers::ice_risk::analyze_ice_risk))
        .route("/api/ice-risk/history", get(handlers::ice_risk::get_ice_analysis_history))
        .route("/api/ice-risk/:tower_id/latest", get(handlers::ice_risk::get_latest_ice_analysis))

        .route("/api/video/:tower_id/verify", post(handlers::video::request_video_verification))
        .route("/api/video/:tower_id/stream", post(handlers::video::start_live_stream))
        .route("/api/video/stream/:session_id", delete(handlers::video::stop_live_stream))
        .route("/api/video/verify/:result_id/manual", post(handlers::video::manual_verify))
        .route("/api/cameras", get(handlers::video::get_cameras))

        .route("/api/shutdown-strategy/:tower_id/evaluate", post(handlers::shutdown_strategy::evaluate_shutdown_strategy))
        .route("/api/shutdown-strategy/:strategy_id/execute", post(handlers::shutdown_strategy::execute_shutdown))
        .route("/api/shutdown-strategy/active", get(handlers::shutdown_strategy::get_active_strategies))
        .route("/api/shutdown-logs", get(handlers::shutdown_strategy::get_shutdown_logs))

        .route("/api/inspections", get(handlers::inspection::get_inspections))
        .route("/api/inspections", post(handlers::inspection::create_inspection))
        .route("/api/inspections/:id", get(handlers::inspection::get_inspection))
        .route("/api/inspections/:tower_id/report", get(handlers::inspection::generate_inspection_report))
        .route("/api/maintenance-tasks", get(handlers::inspection::get_maintenance_tasks))
        .route("/api/maintenance-tasks", post(handlers::inspection::create_maintenance_task))
        .route("/api/maintenance-tasks/:task_id/status", put(handlers::inspection::update_maintenance_task_status))

        .route("/api/weather/impact", get(handlers::weather::analyze_weather_impact))
        .route("/api/weather/forecast", get(handlers::weather::get_weather_forecast))

        .layer(middleware::from_fn_with_state(
            state.clone(),
            handlers::auth::auth_middleware,
        ))
        .with_state(state.clone());

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}
