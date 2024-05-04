use axum::{Extension, Json};
use redis::AsyncCommands;
use serde::Serialize;

use crate::{models::AllDataPoints, AppContext};

// TODO(hito): cache responses

pub async fn datapoints(ctx: Extension<AppContext>) -> Json<AllDataPoints> {
    let mut con = ctx.redis.get_multiplexed_async_connection().await.unwrap();

    let dps: String = con.get(&ctx.config.collector.redis_key).await.unwrap();
    let dps = serde_json::from_str(&dps).unwrap();

    Json(dps)
}

// TODO(hito): something like collector for services, i.e. run a loop that checks every service
// defined in Config.toml every X seconds/minutes and saves it into redis
// then this endpoint fetches it from database on request
pub async fn services(ctx: Extension<AppContext>) -> Json<Vec<ServiceStatus>> {
    let mut services = vec![];

    for service in &ctx.config.status.services {
        let res = reqwest::get(&service.endpoint)
            .await
            .unwrap();

        services.push(ServiceStatus {
            label: service.label.clone(),
            online: res.status().is_success(),
            incident: None,
        });
    }

    Json(services)
}

#[derive(Serialize)]
pub struct ServiceStatus {
    label: String,
    online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    incident: Option<String>,
}
