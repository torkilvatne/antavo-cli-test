use anyhow::{Context, Result, bail};
use http::{HeaderName, HeaderValue};
use serde_json::Value;

use crate::{
    antavo::state::CustomerState,
    config::Config,
    escher::{EscherRequestBuilder, SigningParams, sign_request},
};

pub struct AntavoClient {
    client: reqwest::Client,
    params: SigningParams,
    base_url: String,
    pub customer_id: Option<String>,
}

impl AntavoClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: reqwest::Client::new(),
            params: SigningParams::new(
                config.api_key,
                config.api_secret,
                config.credential_scope,
                "ANTAVO".to_string(),
                "SHA256".to_string(),
                "authorization".to_string(),
                "date".to_string(),
            ),
            base_url: config.base_url,
            customer_id: config.customer_id,
        }
    }

    pub(crate) fn customer_id_required(&self) -> Result<&str> {
        self.customer_id.as_deref().ok_or_else(|| anyhow::anyhow!(
            "No customer ID set. Add ANTAVO_CUSTOMER_ID to .env or pass --customer <id>"
        ))
    }

    /// Send a signed POST to /events and return the response JSON.
    pub async fn post_event(&self, body: Value) -> Result<Value> {
        let url = format!("{}/events", self.base_url);
        let body_bytes = serde_json::to_vec(&body)?;

        let host = reqwest::Url::parse(&url)?
            .host()
            .context("URL has no host")?
            .to_string();

        let path = reqwest::Url::parse(&url)?.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("POST")
            .with_path(&path)
            .with_host(&host)
            .with_body(&body_bytes)
            .add_header("Content-Type", "application/json")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body_bytes)
            .build()?;

        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from /events: {}", status, text);
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Invalid JSON response: {}", text))?;

        Ok(json)
    }

    /// GET /customers/{id} and return the full JSON.
    pub async fn get_customer_raw(&self) -> Result<Value> {
        self.get_customer_raw_for(self.customer_id_required()?).await
    }

    /// Get customer state as a typed struct.
    pub async fn get_customer_state(&self) -> Result<CustomerState> {
        let json = self.get_customer_raw().await?;
        Ok(CustomerState::from_json(&json))
    }

    /// GET /customers/{id}/transactions — optionally filtered by transaction_id.
    pub async fn get_customer_transactions(&self, transaction_id: Option<&str>) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let base = format!("{}/customers/{}/transactions", self.base_url, customer_id);
        let url = match transaction_id {
            Some(tx_id) => format!("{}?id={}", base, tx_id),
            None => base,
        };

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();
        let query = parsed.query().unwrap_or("").to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("GET")
            .with_path(&path)
            .with_host(&host)
            .with_query(&query)
            .with_body(b"")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self.client.get(&url).build()?;
        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from GET /customers/{}/transactions: {}", status, customer_id, text);
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Invalid JSON: {}", text))?;

        Ok(json)
    }

    /// GET /customers/{id}/transactions/{tx_id} — fetch a single transaction by path.
    pub async fn get_transaction(&self, transaction_id: Option<&str>) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let url = match transaction_id {
            Some(tx_id) => format!("{}/customers/{}/transactions/{}", self.base_url, customer_id, tx_id),
            None => format!("{}/customers/{}/transactions", self.base_url, customer_id),
        };

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("GET")
            .with_path(&path)
            .with_host(&host)
            .with_body(b"")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self.client.get(&url).build()?;
        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from GET /customers/{}/transactions: {}", status, customer_id, text);
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Invalid JSON: {}", text))?;

        Ok(json)
    }

    /// GET /customers/{id}/activities/rewards — returns all available rewards for the customer.
    pub async fn get_rewards(&self) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let url = format!("{}/customers/{}/activities/rewards", self.base_url, customer_id);

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("GET")
            .with_path(&path)
            .with_host(&host)
            .with_body(b"")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self.client.get(&url).build()?;
        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from GET /customers/{}/activities/rewards: {}", status, customer_id, text);
        }

        serde_json::from_str(&text).with_context(|| format!("Invalid JSON: {}", text))
    }

    /// POST /customers/{id}/activities/rewards/{reward_id}/claim — claims a reward.
    pub async fn claim_reward(&self, reward_id: &str) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let url = format!(
            "{}/customers/{}/activities/rewards/{}/claim",
            self.base_url, customer_id, reward_id
        );
        let body_bytes = b"{}";

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("POST")
            .with_path(&path)
            .with_host(&host)
            .with_body(body_bytes)
            .add_header("Content-Type", "application/json")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body_bytes.to_vec())
            .build()?;

        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from POST .../rewards/{}/claim: {}", status, reward_id, text);
        }

        serde_json::from_str(&text).with_context(|| format!("Invalid JSON: {}", text))
    }

    /// POST /customers/{id}/activities/rewards/{reward_id}/revoke — revokes a claimed reward.
    pub async fn revoke_reward(&self, reward_id: &str) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let url = format!(
            "{}/customers/{}/activities/rewards/{}/revoke",
            self.base_url, customer_id, reward_id
        );
        let body_bytes = b"{}";

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("POST")
            .with_path(&path)
            .with_host(&host)
            .with_body(body_bytes)
            .add_header("Content-Type", "application/json")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body_bytes.to_vec())
            .build()?;

        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from POST .../rewards/{}/revoke: {}", status, reward_id, text);
        }

        serde_json::from_str(&text).with_context(|| format!("Invalid JSON: {}", text))
    }

    /// GET /customers/{id}/rewards — returns claimed rewards for the customer.
    pub async fn get_claimed_rewards(&self) -> Result<Value> {
        let customer_id = self.customer_id_required()?;
        let url = format!("{}/customers/{}/rewards", self.base_url, customer_id);

        let parsed = reqwest::Url::parse(&url)?;
        let host = parsed.host().context("URL has no host")?.to_string();
        let path = parsed.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("GET")
            .with_path(&path)
            .with_host(&host)
            .with_body(b"")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self.client.get(&url).build()?;
        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from GET /customers/{}/rewards: {}", status, customer_id, text);
        }

        serde_json::from_str(&text).with_context(|| format!("Invalid JSON: {}", text))
    }

    /// Get raw JSON for any customer by ID (used after opt_in to fetch newly created customer).
    pub async fn get_customer_raw_for(&self, customer_id: &str) -> Result<Value> {
        let url = format!("{}/customers/{}", self.base_url, customer_id);

        let host = reqwest::Url::parse(&url)?
            .host()
            .context("URL has no host")?
            .to_string();

        let path = reqwest::Url::parse(&url)?.path().to_string();

        let escher_req = EscherRequestBuilder::new()
            .with_method("GET")
            .with_path(&path)
            .with_host(&host)
            .with_body(b"")
            .build()
            .map_err(|e| anyhow::anyhow!("Escher build error: {}", e))?;

        let signed = sign_request(escher_req, &self.params)
            .map_err(|e| anyhow::anyhow!("Escher sign error: {}", e))?;

        let mut req = self.client.get(&url).build()?;

        let headers = req.headers_mut();
        for (name, value) in signed {
            headers.insert(
                HeaderName::from_bytes(name.as_bytes())
                    .with_context(|| format!("Invalid header name: {}", name))?,
                HeaderValue::from_str(&value)
                    .with_context(|| format!("Invalid header value for {}", name))?,
            );
        }

        let resp = self.client.execute(req).await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            bail!("HTTP {} from GET /customers/{}: {}", status, customer_id, text);
        }

        let json: Value = serde_json::from_str(&text)
            .with_context(|| format!("Invalid JSON: {}", text))?;

        Ok(json)
    }
}
