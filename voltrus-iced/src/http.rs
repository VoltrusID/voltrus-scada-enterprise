use anyhow::Result;
use reqwest::Client;

use crate::models::*;

pub struct HttpClient {
    client: Client,
    base_url: String,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .cookie_store(true)
                .build()
                .expect("failed to build HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&serde_json::json!({
                "username": username,
                "password": password
            }))
            .send()
            .await?;

        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            anyhow::bail!("Login failed: {}", resp.status())
        }
    }

    pub async fn fetch_data(&self) -> Result<Vec<DataSourceData>> {
        let resp = self
            .client
            .get(format!("{}/api/data", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_history(&self, range: &str) -> Result<HistoryResponse> {
        let resp = self
            .client
            .get(format!("{}/api/history?range={}", self.base_url, range))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_data_sources(&self) -> Result<Vec<DataSourceSummary>> {
        let resp = self
            .client
            .get(format!("{}/api/data-sources", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_branding(&self) -> Result<BrandingSettings> {
        let resp = self
            .client
            .get(format!("{}/api/settings/branding", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }
}
