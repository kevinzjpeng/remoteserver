use std::env;
use std::net::IpAddr;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Config {
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub session_secret: String,
    pub encryption_key: String,
    pub database_path: String,
    pub port: String,
    pub base_url: String,
    pub internal_ip: String,
    pub session_duration: Duration,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingRequired(String),
    #[error("Invalid session duration: {0}")]
    InvalidDuration(String),
}

pub fn load_config() -> Result<Config, ConfigError> {
    dotenvy::dotenv().ok();

    let github_client_id = env::var("GITHUB_CLIENT_ID")
        .map_err(|_| ConfigError::MissingRequired("GITHUB_CLIENT_ID".to_string()))?;
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET")
        .map_err(|_| ConfigError::MissingRequired("GITHUB_CLIENT_SECRET".to_string()))?;
    let session_secret = env::var("SESSION_SECRET")
        .map_err(|_| ConfigError::MissingRequired("SESSION_SECRET".to_string()))?;
    let encryption_key = env::var("ENCRYPTION_KEY")
        .map_err(|_| ConfigError::MissingRequired("ENCRYPTION_KEY".to_string()))?;

    let database_path = env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "./data/server-access-manager.db".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let base_url = env::var("BASE_URL").unwrap_or_default();
    let internal_ip = env::var("INTERNAL_IP")
        .unwrap_or_else(|_| detect_internal_ip());

    let github_redirect_uri = env::var("GITHUB_REDIRECT_URI")
        .unwrap_or_else(|_| format!("{}/auth/github/callback", base_url));

    let session_duration = parse_session_duration()?;

    Ok(Config {
        github_client_id,
        github_client_secret,
        github_redirect_uri,
        session_secret,
        encryption_key,
        database_path,
        port,
        base_url,
        internal_ip,
        session_duration,
    })
}

fn parse_session_duration() -> Result<Duration, ConfigError> {
    let duration_str = env::var("SESSION_DURATION").unwrap_or_else(|_| "24h".to_string());

    if let Ok(duration) = humantime::parse_duration(&duration_str) {
        return Ok(duration);
    }

    if let Ok(hours) = duration_str.parse::<u64>() {
        return Ok(Duration::from_secs(hours * 3600));
    }

    Err(ConfigError::InvalidDuration(duration_str))
}

fn detect_internal_ip() -> String {
    String::new()
}

pub fn is_internal_client(client_ip: &str) -> bool {
    if let Ok(ip) = client_ip.parse::<IpAddr>() {
        match ip {
            IpAddr::V4(ipv4) => ipv4.is_loopback() || ipv4.is_private(),
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_load_config_with_defaults() {
        // Save original values
        let orig_id = env::var("GITHUB_CLIENT_ID").ok();
        let orig_secret = env::var("GITHUB_CLIENT_SECRET").ok();
        let orig_session = env::var("SESSION_SECRET").ok();
        let orig_key = env::var("ENCRYPTION_KEY").ok();

        env::set_var("GITHUB_CLIENT_ID", "test_id");
        env::set_var("GITHUB_CLIENT_SECRET", "test_secret");
        env::set_var("SESSION_SECRET", "test_session");
        env::set_var("ENCRYPTION_KEY", "01234567890123456789012345678901");

        let config = load_config().unwrap();
        assert_eq!(config.port, "8080");
        assert_eq!(config.database_path, "./data/server-access-manager.db");
        assert_eq!(config.session_duration.as_secs(), 86400);

        // Restore original values
        restore_env("GITHUB_CLIENT_ID", orig_id);
        restore_env("GITHUB_CLIENT_SECRET", orig_secret);
        restore_env("SESSION_SECRET", orig_session);
        restore_env("ENCRYPTION_KEY", orig_key);
    }

    #[test]
    fn test_is_internal_client() {
        assert!(is_internal_client("127.0.0.1"));
        assert!(is_internal_client("192.168.1.1"));
        assert!(is_internal_client("10.0.0.1"));
        assert!(!is_internal_client("8.8.8.8"));
        assert!(!is_internal_client("invalid"));
    }

    fn restore_env(key: &str, value: Option<String>) {
        match value {
            Some(v) => env::set_var(key, v),
            None => env::remove_var(key),
        }
    }
}
