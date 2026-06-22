use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub clickhouse: ClickHouseConfig,
    pub nats: NatsConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub system: SystemConfig,
    pub video: VideoConfig,
    pub node: NodeConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub host: String,
    pub port: u16,
    pub user: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expire_hours: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemConfig {
    pub ice_alert_threshold_mm: f64,
    pub vibration_alert_threshold_mms: f64,
    pub wind_speed_threshold_ms: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoConfig {
    pub server_url: String,
    pub api_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub node_location: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();

        Ok(Self {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            clickhouse: ClickHouseConfig {
                host: env::var("CLICKHOUSE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("CLICKHOUSE_PORT")
                    .unwrap_or_else(|_| "8123".to_string())
                    .parse()?,
                user: env::var("CLICKHOUSE_USER").unwrap_or_else(|_| "default".to_string()),
                password: env::var("CLICKHOUSE_PASSWORD").unwrap_or_default(),
                database: env::var("CLICKHOUSE_DATABASE")
                    .unwrap_or_else(|_| "cableway_monitor".to_string()),
            },
            nats: NatsConfig {
                host: env::var("NATS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("NATS_PORT")
                    .unwrap_or_else(|_| "4222".to_string())
                    .parse()?,
                user: env::var("NATS_USER").ok(),
                password: env::var("NATS_PASSWORD").ok(),
            },
            redis: RedisConfig {
                host: env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                port: env::var("REDIS_PORT")
                    .unwrap_or_else(|_| "6379".to_string())
                    .parse()?,
                password: env::var("REDIS_PASSWORD").ok(),
            },
            jwt: JwtConfig {
                secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string()),
                expire_hours: env::var("JWT_EXPIRE_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
            },
            system: SystemConfig {
                ice_alert_threshold_mm: env::var("ICE_ALERT_THRESHOLD_MM")
                    .unwrap_or_else(|_| "10.0".to_string())
                    .parse()?,
                vibration_alert_threshold_mms: env::var("VIBRATION_ALERT_THRESHOLD_MMS")
                    .unwrap_or_else(|_| "5.0".to_string())
                    .parse()?,
                wind_speed_threshold_ms: env::var("WIND_SPEED_THRESHOLD_MS")
                    .unwrap_or_else(|_| "25.0".to_string())
                    .parse()?,
            },
            video: VideoConfig {
                server_url: env::var("VIDEO_SERVER_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:8090".to_string()),
                api_key: env::var("VIDEO_API_KEY").unwrap_or_else(|_| "dev-key".to_string()),
            },
            node: NodeConfig {
                node_id: env::var("NODE_ID").unwrap_or_else(|_| "edge-001".to_string()),
                node_location: env::var("NODE_LOCATION").unwrap_or_else(|_| "景区入口".to_string()),
            },
        })
    }

    pub fn clickhouse_url(&self) -> String {
        format!(
            "http://{}:{}",
            self.clickhouse.host, self.clickhouse.port
        )
    }

    pub fn nats_url(&self) -> String {
        format!("nats://{}:{}", self.nats.host, self.nats.port)
    }

    pub fn redis_url(&self) -> String {
        if let Some(password) = &self.redis.password {
            format!("redis://:{}@{}:{}", password, self.redis.host, self.redis.port)
        } else {
            format!("redis://{}:{}", self.redis.host, self.redis.port)
        }
    }
}
