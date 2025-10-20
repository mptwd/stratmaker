use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::errors::AppError;

#[derive(Deserialize, Debug)]
pub struct DatasetMeta {
    pub asset: String,
    pub timeframe: String,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub count: usize,
    pub version: i32,
    pub path: String,
    pub ta: Vec<String>,
}

#[derive(Clone)]
pub struct DatasetManagerClient {
    pub http: Client,
    pub base_url: String,
}

impl DatasetManagerClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into(),
        }
    }

    pub async fn get_dataset(&self, name: String) -> Result<DatasetMeta, AppError> {
        get_dataset_metadata(&self.http, &self.base_url, &name).await
    }
}

async fn get_dataset_metadata(
    client: &Client,
    base_url: &str,
    name: &str,
) -> Result<DatasetMeta, AppError> {
    let url = format!("{}/datasets/{}", base_url, name);
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<DatasetMeta>().await {
                Ok(r) => Ok(r),
                Err(e) => Err(AppError::Reqwest(e)),
            }
        }
        Ok(_) => {
            Err(AppError::DatasetNotFound)
        }
        Err(e) => {
            Err(AppError::Reqwest(e))
        }
    }
}
