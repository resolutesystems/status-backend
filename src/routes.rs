use axum::{extract::Path, Extension, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{models::{DataPoints, Incident}, AppContext};

// TODO(hito): cache responses
pub async fn datapoints(ctx: Extension<AppContext>) -> Json<Vec<DataPoints>> {
    let datapoints = sqlx::query_as!(DataPoints, "SELECT * FROM (SELECT * FROM datapoints ORDER BY id DESC LIMIT 5) subquery ORDER BY id")
        .fetch_all(&ctx.db)
        .await
        .unwrap();
    Json(datapoints)
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

        let row = sqlx::query_as!(Incident, "SELECT * FROM incidents WHERE service = $1", service.label)
            .fetch_optional(&ctx.db)
            .await
            .unwrap();

        let incident = match row {
            Some(row) => Some(row.message),
            None => None
        };

        services.push(ServiceStatus {
            label: service.label.clone(),
            online: res.status().is_success(),
            incident,
        });
    }

    Json(services)
}

pub async fn add_incident(ctx: Extension<AppContext>, body: Json<AddIncidentBody>) {
    let row = sqlx::query!("SELECT * FROM incidents WHERE service = $1", body.service)
        .fetch_optional(&ctx.db)
        .await
        .unwrap();

    if let Some(_) = row {
        // TODO(hito): return error that this incident already exists
    }

    sqlx::query!("INSERT INTO incidents (service, message) VALUES ($1, $2)", body.service, body.message)
        .execute(&ctx.db)
        .await
        .unwrap();
}

pub async fn delete_incident(ctx: Extension<AppContext>, Path(service): Path<String>) -> StatusCode {
    sqlx::query!("DELETE FROM incidents WHERE service = $1", service)
        .execute(&ctx.db)
        .await
        .unwrap();

    StatusCode::NO_CONTENT
}

#[derive(Serialize)]
pub struct ServiceStatus {
    label: String,
    online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    incident: Option<String>,
}

#[derive(Deserialize)]
pub struct AddIncidentBody {
    service: String,
    message: String,
}
