pub mod config;
pub mod db;
pub mod models;
pub mod handlers;
pub mod services;
pub mod mq;
pub mod websocket;
pub mod routes;

use config::AppConfig;
use std::sync::Arc;
use tokio::sync::broadcast;
use std::time::Instant;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub config: AppConfig,
    pub clickhouse_client: db::clickhouse::ClickHouseClient,
    pub nats_client: mq::NatsClient,
    pub redis_client: std::sync::Arc<redis::aio::ConnectionManager>,
    pub alert_tx: broadcast::Sender<models::alert::AlertMessage>,
    pub start_time: Instant,
}

impl AppStateInner {
    pub async fn new(config: AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let (alert_tx, _) = broadcast::channel(1000);

        let clickhouse_client =
            db::clickhouse::ClickHouseClient::new(config.clickhouse_url(), config.clickhouse.database.clone())
                .await?;

        let nats_client = mq::NatsClient::new(config.nats_url()).await?;

        let redis_client = redis::Client::open(config.redis_url())?;
        let redis_manager = redis::aio::ConnectionManager::new(redis_client).await?;

        Ok(Self {
            config,
            clickhouse_client,
            nats_client,
            redis_client: std::sync::Arc::new(redis_manager),
            alert_tx,
            start_time: Instant::now(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("NotFound: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Database(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Nats(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Redis(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AppError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            AppError::Authentication(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (axum::http::StatusCode::FORBIDDEN, msg),
            AppError::Authorization(msg) => (axum::http::StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ExternalService(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg),
        };

        let body = serde_json::json!({
            "error": {
                "code": status.as_u16(),
                "message": message
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

impl From<clickhouse::error::Error> for AppError {
    fn from(err: clickhouse::error::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<async_nats::Error> for AppError {
    fn from(err: async_nats::Error) -> Self {
        AppError::Nats(err.to_string())
    }
}
