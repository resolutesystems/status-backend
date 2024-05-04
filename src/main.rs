mod routes;
mod models;
mod collector;
mod config;

#[cfg(not(unix))]
use std::future;

use axum::{routing::get, Extension, Router};
use config::Config;
use tokio::{net::TcpListener, signal};

use crate::{config::load_config, routes::{datapoints, services}};

#[derive(Clone)]
struct AppContext {
    config: Config,
    redis: redis::Client,
}

async fn start_api(config: Config, redis: redis::Client) -> anyhow::Result<()> {
    let ctx = AppContext { config: config.clone(), redis };

    let router = Router::new()
        .route("/datapoints", get(datapoints))
        .route("/services", get(services))
        .layer(Extension(ctx));

    let listener = TcpListener::bind(&config.api.bind).await?;
    println!("api is running on http://{}", config.api.bind);
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await?;
    let redis = redis::Client::open(config.redis.url.clone())?;

    tokio::select! {
        t = start_api(config.clone(), redis.clone()) => t?,
        t = collector::start(config.collector, redis) => t?,
    }

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
    let terminate = future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
