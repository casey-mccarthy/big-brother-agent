use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub api_url: Option<String>,

    #[serde(default = "default_interval")]
    pub interval_seconds: u64,

    #[serde(default)]
    pub tls_insecure: bool,
}

fn default_interval() -> u64 {
    1800
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: None,
            interval_seconds: default_interval(),
            tls_insecure: false,
        }
    }
}

/// Get the directory containing the executable
pub fn exe_dir() -> Result<PathBuf> {
    let exe_path = std::env::current_exe().context("failed to get executable path")?;
    exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("executable has no parent directory"))
}

/// Generate template config.toml if it doesn't exist
fn generate_template_config(config_path: &PathBuf) -> Result<()> {
    let template = r#"# Inventory Agent Configuration
#
# This file configures the endpoint inventory agent service.
# Environment variables override these settings:
#   - INVENTORY_API_URL
#   - INVENTORY_INTERVAL_SECONDS
#   - INVENTORY_TLS_INSECURE

# REQUIRED: API endpoint for check-ins
# Example: api_url = "https://inventory-server.example.com:8443/checkin"
api_url = "http://localhost:8443/checkin"

# Check-in interval in seconds (default: 1800 = 30 minutes)
interval_seconds = 1800

# Accept invalid TLS certificates (LAB USE ONLY - do not enable in production)
tls_insecure = false
"#;

    std::fs::write(config_path, template).with_context(|| {
        format!(
            "failed to write template config to {}",
            config_path.display()
        )
    })?;

    println!("Generated template config file: {}", config_path.display());
    Ok(())
}

/// Load config from config.toml in the same directory as the executable.
/// Generates a template file if it doesn't exist.
/// Environment variables override config file values.
pub fn load_config() -> Result<Config> {
    let exe_dir = exe_dir()?;
    let config_path = exe_dir.join("config.toml");

    // Load config from file or use defaults
    let mut config = if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read config file: {}", config_path.display()))?;
        toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file: {}", config_path.display()))?
    } else {
        // Auto-generate template config file
        generate_template_config(&config_path)?;
        Config::default()
    };

    // Environment variables override config file
    if let Ok(url) = std::env::var("INVENTORY_API_URL") {
        config.api_url = Some(url);
    }

    if let Ok(interval) = std::env::var("INVENTORY_INTERVAL_SECONDS") {
        if let Ok(val) = interval.parse::<u64>() {
            config.interval_seconds = val;
        }
    }

    if let Ok(insecure) = std::env::var("INVENTORY_TLS_INSECURE") {
        config.tls_insecure = insecure.eq_ignore_ascii_case("true");
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.api_url, None);
        assert_eq!(config.interval_seconds, 1800);
        assert_eq!(config.tls_insecure, false);
    }

    #[test]
    fn test_default_interval() {
        assert_eq!(default_interval(), 1800);
    }

    #[test]
    fn test_toml_parse_minimal() {
        let toml = r#"api_url = "http://test:8080/checkin""#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.api_url, Some("http://test:8080/checkin".to_string()));
        assert_eq!(config.interval_seconds, 1800);
        assert_eq!(config.tls_insecure, false);
    }

    #[test]
    fn test_toml_parse_full() {
        let toml = r#"
            api_url = "https://server:8443/checkin"
            interval_seconds = 3600
            tls_insecure = true
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(
            config.api_url,
            Some("https://server:8443/checkin".to_string())
        );
        assert_eq!(config.interval_seconds, 3600);
        assert_eq!(config.tls_insecure, true);
    }

    #[test]
    fn test_toml_parse_partial() {
        let toml = r#"
            interval_seconds = 900
        "#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.api_url, None);
        assert_eq!(config.interval_seconds, 900);
        assert_eq!(config.tls_insecure, false);
    }

    #[test]
    fn test_exe_dir_success() {
        let result = exe_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_toml_invalid_type() {
        // Test that invalid type for interval_seconds field fails gracefully
        let toml = r#"
            interval_seconds = "not a number"
        "#;
        let result: Result<Config, toml::de::Error> = toml::from_str(toml);
        assert!(result.is_err());
    }
}
