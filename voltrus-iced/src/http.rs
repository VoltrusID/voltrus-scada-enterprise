use std::sync::Arc;

use anyhow::Result;
use reqwest::cookie::Jar;
use reqwest::Client;

use crate::models::*;

pub struct HttpClient {
    client: Client,
    base_url: String,
    cookie_jar: Arc<Jar>,
}

impl HttpClient {
    pub fn new(base_url: &str) -> Self {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            cookie_jar: jar,
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

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self {
            client: Client::builder()
                .cookie_provider(self.cookie_jar.clone())
                .build()
                .expect("failed to build HTTP client"),
            base_url: self.base_url.clone(),
            cookie_jar: self.cookie_jar.clone(),
        }
    }
}
