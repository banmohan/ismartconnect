use std::time::{Duration, Instant};

use axum::http::{HeaderValue, StatusCode};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::error::ApiError;
use crate::iso::iso_from_message;
use crate::logging;
use crate::text::{truncate, LOG_BODY_MAX};

#[derive(Clone)]
pub struct CbsClient {
    http: Client,
    base_url: String,
    api_key: HeaderValue,
}

impl CbsClient {
    pub fn new(base_url: &str, api_key: &str, timeout_secs: u64) -> anyhow::Result<Self> {
        let api_key = HeaderValue::from_str(api_key.trim())
            .map_err(|_| anyhow::anyhow!("CBS_API_KEY is not a valid HTTP header value"))?;
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("mobile-api")
            .build()?;
        Ok(Self {
            http,
            base_url: base_url.trim().trim_end_matches('/').to_string(),
            api_key,
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str, tenant: &str) -> Result<T, ApiError> {
        let started = Instant::now();
        let response = self
            .http
            .get(self.url(path))
            .header("x-api-key", self.api_key.clone())
            .header("tenant", tenant_header(tenant)?)
            .send()
            .await;
        self.finish(path, started, response).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        tenant: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let started = Instant::now();
        let response = self
            .http
            .post(self.url(path))
            .header("x-api-key", self.api_key.clone())
            .header("tenant", tenant_header(tenant)?)
            .json(body)
            .send()
            .await;
        self.finish(path, started, response).await
    }

    async fn finish<T: DeserializeOwned>(
        &self,
        path: &str,
        started: Instant,
        response: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<T, ApiError> {
        let elapsed = started.elapsed().as_millis().min(i32::MAX as u128) as i32;
        let response = match response {
            Ok(response) => response,
            Err(err) => {
                let message = err.to_string();
                logging::set_upstream(path, 0, elapsed, &message);
                return Err(ApiError::transport(message));
            }
        };
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        logging::set_upstream(path, status.as_u16() as i32, elapsed, &truncate(&text, LOG_BODY_MAX));
        if status == StatusCode::OK {
            serde_json::from_str(&text).map_err(|err| {
                ApiError::mapped(format!("Cannot parse the json value to model: {err}"), "96")
            })
        } else {
            Err(map_status(status, &text))
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path.trim_start_matches('/'))
    }
}

fn tenant_header(tenant: &str) -> Result<HeaderValue, ApiError> {
    HeaderValue::from_str(tenant).map_err(|_| ApiError::mapped("invalid tenant header", "30"))
}

fn map_status(status: StatusCode, body: &str) -> ApiError {
    let message = match status.as_u16() {
        400 | 500 => json_message(body).unwrap_or_else(|| reason(status)),
        401 => "Unauthorized".to_string(),
        404 => "content not found".to_string(),
        _ => json_message(body).unwrap_or_else(|| reason(status)),
    };
    let iso = iso_from_message(&message);
    ApiError::mapped(message, iso)
}

fn reason(status: StatusCode) -> String {
    status.canonical_reason().unwrap_or("Unknown Error").to_string()
}

pub fn json_message(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    value
        .get("message")
        .or_else(|| value.get("Message"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|message| !message.is_empty())
}
