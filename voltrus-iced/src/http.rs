use std::sync::Arc;

use anyhow::Result;
use reqwest::cookie::Jar;
use reqwest::Client;

use crate::models::*;

pub struct HttpClient {
    client: Client,
    pub base_url: String,
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

    // -- Auth --

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResponse> {
        let resp = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&LoginRequest {
                username: username.into(),
                password: password.into(),
            })
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            anyhow::bail!("Login failed: {}", resp.status())
        }
    }

    pub async fn logout(&self) -> Result<()> {
        let _ = self
            .client
            .post(format!("{}/api/auth/logout", self.base_url))
            .send()
            .await?;
        Ok(())
    }

    pub async fn auth_me(&self) -> Result<AuthMeResponse> {
        let resp = self
            .client
            .get(format!("{}/api/auth/me", self.base_url))
            .send()
            .await?;
        if resp.status().is_success() {
            Ok(resp.json().await?)
        } else {
            anyhow::bail!("Not authenticated")
        }
    }

    // -- Data Sources / Sensors --

    pub async fn fetch_data_sources(&self) -> Result<Vec<SensorConfig>> {
        let resp = self
            .client
            .get(format!("{}/api/data-sources", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn fetch_screen_data_sources(&self, screen_id: &str) -> Result<Vec<SensorConfig>> {
        if screen_id == "all" {
            return self.fetch_data_sources().await;
        }
        let resp = self
            .client
            .get(format!(
                "{}/api/screens/{}/data-sources",
                self.base_url, screen_id
            ))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn create_sensor(&self, sensor: &SensorConfig) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}/api/data-sources", self.base_url))
            .json(sensor)
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn update_sensor(&self, sensor: &SensorConfig) -> Result<()> {
        self.client
            .put(format!(
                "{}/api/data-sources/{}",
                self.base_url, sensor.id
            ))
            .json(sensor)
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_sensor(&self, id: i64) -> Result<()> {
        self.client
            .delete(format!("{}/api/data-sources/{}", self.base_url, id))
            .send()
            .await?;
        Ok(())
    }

    pub async fn add_sensor_to_screen(&self, screen_id: &str, sensor_id: i64) -> Result<()> {
        self.client
            .post(format!(
                "{}/api/screens/{}/data-sources",
                self.base_url, screen_id
            ))
            .json(&serde_json::json!({ "data_source_id": sensor_id }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn remove_sensor_from_screen(
        &self,
        screen_id: &str,
        sensor_id: i64,
    ) -> Result<()> {
        self.client
            .delete(format!(
                "{}/api/screens/{}/data-sources/{}",
                self.base_url, screen_id, sensor_id
            ))
            .send()
            .await?;
        Ok(())
    }

    // -- Live Data --

    pub async fn fetch_data(&self) -> Result<Vec<SensorData>> {
        let resp = self
            .client
            .get(format!("{}/api/data", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    /// Build a reqwest Request for SSE streaming. The caller consumes the
    /// response body as a stream.
    pub fn sse_request(&self) -> Result<reqwest::Request> {
        let req = self
            .client
            .get(format!("{}/api/stream", self.base_url))
            .build()?;
        Ok(req)
    }

    pub fn client_clone(&self) -> Client {
        Client::builder()
            .cookie_provider(self.cookie_jar.clone())
            .build()
            .expect("failed to build HTTP client")
    }

    // -- History --

    pub async fn fetch_history(&self, range: &str) -> Result<HistoryResponse> {
        let resp = self
            .client
            .get(format!("{}/api/history?range={}", self.base_url, range))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    // -- Screens --

    pub async fn fetch_screens(&self) -> Result<Vec<Screen>> {
        let resp = self
            .client
            .get(format!("{}/api/screens", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn create_screen(&self, name: &str) -> Result<Screen> {
        let resp = self
            .client
            .post(format!("{}/api/screens", self.base_url))
            .json(&CreateScreenRequest {
                name: name.into(),
            })
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    pub async fn rename_screen(&self, id: &str, name: &str) -> Result<()> {
        self.client
            .put(format!("{}/api/screens/{}", self.base_url, id))
            .json(&RenameScreenRequest {
                name: name.into(),
            })
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_screen(&self, id: &str) -> Result<()> {
        self.client
            .delete(format!("{}/api/screens/{}", self.base_url, id))
            .send()
            .await?;
        Ok(())
    }

    // -- Alarms --

    pub async fn fetch_alarms(&self) -> Result<Vec<Alarm>> {
        let resp = self
            .client
            .get(format!("{}/api/alarms", self.base_url))
            .send()
            .await?;
        Ok(resp.json().await?)
    }

    // -- Branding --

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
            client: self.client_clone(),
            base_url: self.base_url.clone(),
            cookie_jar: self.cookie_jar.clone(),
        }
    }
}
