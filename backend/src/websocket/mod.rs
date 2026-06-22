use crate::{AppState, AppError, models::*, mq};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::HeaderMap,
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, error, warn};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    Subscribe {
        channel: String,
        tower_id: Option<Uuid>,
    },
    Unsubscribe {
        channel: String,
        tower_id: Option<Uuid>,
    },
    Alert {
        alert: alert::AlertMessage,
    },
    TowerStatus {
        status: tower::TowerStatusResponse,
    },
    SensorData {
        tower_id: Uuid,
        data_type: String,
        data: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Error {
        message: String,
    },
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let user_agent = headers.get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, state, user_agent))
}

async fn handle_socket(socket: WebSocket, state: AppState, user_agent: Option<String>) {
    let (mut sender, mut receiver) = socket.split();

    info!("New WebSocket connection established");

    let mut alert_rx = state.alert_tx.subscribe();
    let (tx, mut internal_rx) = tokio::sync::mpsc::channel::<WebSocketMessage>(100);
    let mut subscriptions: Vec<String> = Vec::new();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = internal_rx.recv().await {
            let text = match serde_json::to_string(&msg) {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to serialize WebSocket message: {}", e);
                    continue;
                }
            };
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    let alert_task = {
        let tx = tx.clone();
        let state_clone = state.clone();
        tokio::spawn(async move {
            loop {
                match alert_rx.recv().await {
                    Ok(alert) => {
                        if subscriptions.iter().any(|s| s == "alerts" || s == &format!("alerts:{}", alert.tower_id)) {
                            if tx.send(WebSocketMessage::Alert { alert }).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        warn!("Alert channel lagged, some messages may be missed");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        })
    };

    let heartbeat_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                if tx.send(WebSocketMessage::Heartbeat {
                    timestamp: Utc::now(),
                }).await.is_err() {
                    break;
                }
            }
        })
    };

    let receive_task = async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<WebSocketMessage>(&text) {
                        Ok(WebSocketMessage::Subscribe { channel, tower_id }) => {
                            let channel_key = if let Some(tower_id) = tower_id {
                                format!("{}:{}", channel, tower_id)
                            } else {
                                channel.clone()
                            };
                            if !subscriptions.contains(&channel_key) {
                                subscriptions.push(channel_key.clone());
                                info!("Subscribed to channel: {}", channel_key);
                            }
                        }
                        Ok(WebSocketMessage::Unsubscribe { channel, tower_id }) => {
                            let channel_key = if let Some(tower_id) = tower_id {
                                format!("{}:{}", channel, tower_id)
                            } else {
                                channel.clone()
                            };
                            subscriptions.retain(|s| s != &channel_key);
                            info!("Unsubscribed from channel: {}", channel_key);
                        }
                        Ok(_) => {
                            warn!("Received unexpected message type from client");
                        }
                        Err(e) => {
                            warn!("Invalid WebSocket message: {}", e);
                            let _ = tx.send(WebSocketMessage::Error {
                                message: format!("Invalid message: {}", e),
                            }).await;
                        }
                    }
                }
                Message::Binary(_) => {
                    warn!("Binary messages not supported");
                }
                Message::Close(_) => {
                    info!("WebSocket close message received");
                    break;
                }
                Message::Ping(data) => {
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Message::Pong(_) => {}
            }
        }
    };

    tokio::select! {
        _ = send_task => {}
        _ = alert_task => {}
        _ = heartbeat_task => {}
        _ = receive_task => {}
    }

    info!("WebSocket connection closed");
}

pub async fn broadcast_tower_status(
    state: &AppState,
    status: tower::TowerStatusResponse,
) -> Result<(), AppError> {
    use serde::Serialize;

    let msg = WebSocketMessage::TowerStatus {
        status: status.clone(),
    };
    let payload = serde_json::to_string(&msg)?;

    let _ = mq::publisher::publish_tower_status_updated(state, &status).await;

    Ok(())
}

pub async fn broadcast_sensor_data(
    state: &AppState,
    tower_id: Uuid,
    data_type: String,
    data: serde_json::Value,
) -> Result<(), AppError> {
    let msg = WebSocketMessage::SensorData {
        tower_id,
        data_type,
        data,
        timestamp: Utc::now(),
    };
    let _payload = serde_json::to_string(&msg)?;

    Ok(())
}
