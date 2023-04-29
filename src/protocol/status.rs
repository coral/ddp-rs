use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub man: Option<String>,
    #[serde(rename = "mod")]
    pub model: Option<String>,
    pub ver: Option<String>,
    pub mac: Option<String>,
    pub push: Option<bool>,
    pub ntp: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusResponse {
    Status(DeviceStatus),
    Update { update: String, state: String },
}
