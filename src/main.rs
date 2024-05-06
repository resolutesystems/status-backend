mod routes;
mod models;
mod collector;
mod config;

#[cfg(not(unix))]
use std::future;

use axum::{routing::{delete, get, post}, Extension, Router};
use config::Config;
use dotenvy_macro::dotenv;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::{net::TcpListener, signal};

use crate::{config::load_config, routes::{add_incident, datapoints, delete_incident, services}};

const DATABASE_URL: &str = dotenv!("DATABASE_URL");

#[derive(Clone)]
struct AppContext {
    config: Config,
    db: PgPool,
}

async fn start_api(config: Config, db: PgPool) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&config.api.bind).await?;
    println!("api is running on http://{}", config.api.bind);

    let ctx = AppContext { config, db };

    let router = Router::new()
        .route("/datapoints", get(datapoints))
        .route("/services", get(services))
        .route("/incidents", post(add_incident))
        .route("/incidents/:service", delete(delete_incident))
        .layer(Extension(ctx));

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config().await?;
    let db = PgPoolOptions::new()
        .max_connections(2)
        .connect(DATABASE_URL)
        .await?;

    tokio::select! {
        t = start_api(config.clone(), db.clone()) => t?,
        t = collector::start(config.collector, db) => t?,
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
