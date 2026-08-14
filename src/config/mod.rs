/// Configuration management for the application
/// 
/// Handles loading TOML configuration files with defaults fallback and validation.

pub mod animation;

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub animation: animation::AnimationConfig,
}

impl Config {
    /// Load configuration from file with defaults fallback
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            warn!("Config file not found at {:?}, using defaults", config_path);
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file path
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = toml::from_str(&content)?;
        config.validate()?;
        
        info!("Configuration loaded from {:?}", path);
        Ok(config)
    }

    /// Get the default configuration directory
    fn config_dir() -> PathBuf {
        let home = std::env::var("HOME")
            .unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config/oled-wallpaper")
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<()> {
        self.animation.validate()?;
        Ok(())
    }

    /// Create a configuration with defaults
    fn default() -> Self {
        Config {
            animation: animation::AnimationConfig::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            animation: animation::AnimationConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.animation.planet_speed > 0.0);
    }

    #[test]
    fn test_config_validation_passes_for_default() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
}
