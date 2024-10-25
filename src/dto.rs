use std::collections::HashMap;

use serde_derive::Deserialize;
use serde_derive::Serialize;
use warp::reject::Rejection;

use crate::rejection::Error;
use crate::utils::is_valid_field;
use crate::utils::is_valid_key_field;

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

impl CreateUserRequest {
    pub fn validate(&self) -> Result<(), Rejection> {
        if !is_valid_key_field(&self.username) {
            return Err(warp::reject::custom(Error::InvalidField(
                "username".to_string(),
            )));
        }
        if !is_valid_field(&self.password) {
            return Err(warp::reject::custom(Error::InvalidField(
                "password".to_string(),
            )));
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Progress {
    pub document: String,
    pub progress: String,
    pub percentage: f64,
    pub device: String,
    pub device_id: String,
    pub timestamp: Option<u64>,
}

impl Progress {
    pub fn to_vec(&self) -> Vec<(&str, String)> {
        vec![
            ("document", self.document.clone()),
            ("progress", self.progress.clone()),
            ("percentage", self.percentage.to_string()),
            ("device", self.device.clone()),
            ("device_id", self.device_id.clone()),
            ("timestamp", self.timestamp.unwrap_or_default().to_string()),
        ]
    }

    pub fn validate(&self) -> Result<(), Rejection> {
        if !is_valid_key_field(&self.document) {
            return Err(warp::reject::custom(Error::InvalidField(
                "document".to_string(),
            )));
        }
        if self.percentage < 0.0 || self.percentage > 100.0 {
            return Err(warp::reject::custom(Error::InvalidField(
                "percentage".to_string(),
            )));
        }
        if self.progress.is_empty() {
            return Err(warp::reject::custom(Error::InvalidField(
                "progress".to_string(),
            )));
        }
        if self.device.is_empty() {
            return Err(warp::reject::custom(Error::InvalidField(
                "device".to_string(),
            )));
        }
        Ok(())
    }
}

impl From<HashMap<String, String>> for Progress {
    fn from(map: HashMap<String, String>) -> Self {
        Self {
            document: safe_get(&map, "document"),
            progress: safe_get(&map, "progress"),
            percentage: safe_get(&map, "percentage").parse::<f64>().unwrap_or(0.0),
            device: safe_get(&map, "device"),
            device_id: safe_get(&map, "device_id"),
            timestamp: Some(safe_get(&map, "timestamp").parse::<u64>().unwrap_or(0)),
        }
    }
}

fn safe_get(map: &HashMap<String, String>, key: &str) -> String {
    map.get(key).unwrap_or(&String::new()).to_string()
}
