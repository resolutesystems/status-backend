use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DataSet {
    pub label: String,
    pub data: Vec<f32>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct DataPoints {
    pub timestamps: Vec<DateTime<Utc>>,
    pub datasets: Vec<DataSet>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct AllDataPoints {
    pub cpu: DataPoints,
    pub memory: DataPoints,
    pub network: DataPoints,
}
