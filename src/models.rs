use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DataPoints {
    pub id: i64,
    pub cpu: f32,
    pub memory: f32,
    pub transmitted: f64,
    pub received: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct Incident {
    pub id: i64,
    pub service: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}
