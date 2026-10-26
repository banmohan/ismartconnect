use anyhow::{bail, Context};

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub cbs_account_url: String,
    pub cbs_api_key: String,
    pub admin_username: Option<String>,
    pub admin_password: Option<String>,
    pub cookie_secret: String,
    pub cookie_secure: bool,
    pub bind_addr: String,
    pub cbs_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let admin_username = optional("ADMIN_USERNAME");
        let admin_password = optional("ADMIN_PASSWORD");
        if admin_username.is_some() != admin_password.is_some() {
            bail!("set both ADMIN_USERNAME and ADMIN_PASSWORD, or neither");
        }
        let timeout = std::env::var("CBS_TIMEOUT_SECS")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| value.parse::<u64>())
            .transpose()
            .context("CBS_TIMEOUT_SECS must be a number of seconds")?
            .unwrap_or(60);
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            cbs_account_url: required("CBS_ACCOUNT_URL")?,
            cbs_api_key: required("CBS_API_KEY")?,
            admin_username,
            admin_password,
            cookie_secret: required("COOKIE_SECRET")?,
            cookie_secure: std::env::var("COOKIE_SECURE").is_ok_and(|value| {
                matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes")
            }),
            bind_addr: optional("BIND_ADDR").unwrap_or_else(|| "0.0.0.0:8080".to_string()),
            cbs_timeout_secs: timeout,
        })
    }
}

fn required(name: &str) -> anyhow::Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => bail!("missing required environment variable {name}"),
    }
}

fn optional(name: &str) -> Option<String> {
    std::env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}
