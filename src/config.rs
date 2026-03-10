use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub credential_scope: String,
    pub customer_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            base_url: std::env::var("ANTAVO_BASE_URL")
                .context("ANTAVO_BASE_URL not set")?,
            api_key: std::env::var("ANTAVO_API_KEY")
                .context("ANTAVO_API_KEY not set")?,
            api_secret: std::env::var("ANTAVO_API_SECRET")
                .context("ANTAVO_API_SECRET not set")?,
            credential_scope: std::env::var("ANTAVO_CREDENTIAL_SCOPE")
                .context("ANTAVO_CREDENTIAL_SCOPE not set")?,
            customer_id: std::env::var("ANTAVO_CUSTOMER_ID").ok(),
        })
    }
}
