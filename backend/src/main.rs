use cableway_monitor::*;
use clap::Parser;
use std::net::SocketAddr;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    migrate: bool,

    #[arg(short, long, default_value_t = false)]
    init_data: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cableway_monitor=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::AppConfig::load()?;
    tracing::info!("Loaded configuration for node: {}", config.node.node_id);

    let state = AppStateInner::new(config.clone()).await?;
    let app_state = std::sync::Arc::new(state);

    if args.migrate {
        tracing::info!("Running database migrations...");
        app_state.clickhouse_client.init_database().await?;
        app_state.clickhouse_client.create_tables().await?;
        tracing::info!("Migrations completed successfully");
    }

    if args.init_data {
        tracing::info!("Initializing sample data...");
        services::data_init::init_sample_data(app_state.clone()).await?;
        tracing::info!("Sample data initialized");
    }

    let app = routes::create_router(app_state.clone());

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!("Server starting on http://{}", addr);

    let data_collection_handle = tokio::spawn(async move {
        if let Err(e) = services::data_collection::start_data_collection(app_state.clone()).await {
            tracing::error!("Data collection service error: {}", e);
        }
    });

    let nats_subscriber_handle = tokio::spawn(async move {
        if let Err(e) = mq::subscriber::start_subscribers(app_state.clone()).await {
            tracing::error!("NATS subscriber error: {}", e);
        }
    });

    let server_handle = axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal());

    tokio::select! {
        _ = server_handle => {
            tracing::info!("Server shut down");
        }
        _ = data_collection_handle => {
            tracing::info!("Data collection service stopped");
        }
        _ = nats_subscriber_handle => {
            tracing::info!("NATS subscriber stopped");
        }
    }

    tracing::info!("Application shutdown complete");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        }
    }
}
