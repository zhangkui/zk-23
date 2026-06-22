use crate::AppError;
use async_nats::Client;
use std::sync::Arc;

#[derive(Clone)]
pub struct NatsClient {
    client: Arc<Client>,
}

impl NatsClient {
    pub async fn new(url: String) -> Result<Self, AppError> {
        let client = async_nats::connect(url).await?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub fn get_client(&self) -> Arc<Client> {
        self.client.clone()
    }

    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), AppError> {
        self.client
            .publish(subject.to_string(), payload.into())
            .await
            .map_err(|e| AppError::Nats(e.to_string()))?;
        Ok(())
    }

    pub async fn request(
        &self,
        subject: &str,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<Vec<u8>, AppError> {
        let response = self
            .client
            .request(subject.to_string(), payload.into())
            .await
            .map_err(|e| AppError::Nats(e.to_string()))?;
        Ok(response.payload.to_vec())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber, AppError> {
        let subscriber = self
            .client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| AppError::Nats(e.to_string()))?;
        Ok(subscriber)
    }

    pub async fn queue_subscribe(
        &self,
        subject: &str,
        queue_group: &str,
    ) -> Result<async_nats::Subscriber, AppError> {
        let subscriber = self
            .client
            .queue_subscribe(subject.to_string(), queue_group.to_string())
            .await
            .map_err(|e| AppError::Nats(e.to_string()))?;
        Ok(subscriber)
    }
}

pub mod subjects {
    pub const VIBRATION_DATA: &str = "sensor.vibration";
    pub const WIND_SPEED_DATA: &str = "sensor.wind_speed";
    pub const ICE_DETECTION_DATA: &str = "sensor.ice_detection";
    pub const WEATHER_DATA: &str = "sensor.weather";
    pub const ALERT_TRIGGERED: &str = "alert.triggered";
    pub const ALERT_ACKNOWLEDGED: &str = "alert.acknowledged";
    pub const ALERT_RESOLVED: &str = "alert.resolved";
    pub const SHUTDOWN_STRATEGY_TRIGGERED: &str = "shutdown.triggered";
    pub const SHUTDOWN_EXECUTED: &str = "shutdown.executed";
    pub const VIDEO_VERIFICATION_REQUESTED: &str = "video.verification.requested";
    pub const VIDEO_VERIFICATION_COMPLETED: &str = "video.verification.completed";
    pub const INSPECTION_CREATED: &str = "inspection.created";
    pub const INSPECTION_COMPLETED: &str = "inspection.completed";
    pub const TOWER_STATUS_UPDATED: &str = "tower.status.updated";
    pub const SYSTEM_HEARTBEAT: &str = "system.heartbeat";
    pub const DATA_INGESTION_COMPLETED: &str = "data.ingestion.completed";

    pub fn tower_specific(prefix: &str, tower_id: &uuid::Uuid) -> String {
        format!("{}.{}", prefix, tower_id)
    }
}
