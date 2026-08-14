/// Animation configuration parameters
/// 
/// Loads and validates animation-specific settings from configuration files.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Animation configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfig {
    #[serde(default = "default_planet_speed")]
    pub planet_speed: f32,

    #[serde(default = "default_planet_colors")]
    pub planet_colors: Vec<[f32; 4]>,

    #[serde(default = "default_planet_sizes")]
    pub planet_sizes: Vec<f32>,

    #[serde(default = "default_sun_color")]
    pub sun_color: [f32; 4],

    #[serde(default = "default_sun_size")]
    pub sun_size: f32,
}

fn default_planet_speed() -> f32 {
    1.0
}

fn default_planet_colors() -> Vec<[f32; 4]> {
    vec![
        [0.2, 0.6, 1.0, 1.0], // blue planet
        [1.0, 0.4, 0.2, 1.0], // red/orange planet
    ]
}

fn default_planet_sizes() -> Vec<f32> {
    vec![0.8, 0.5]
}

fn default_sun_color() -> [f32; 4] {
    [1.0, 0.9, 0.0, 1.0] // yellow sun
}

fn default_sun_size() -> f32 {
    2.0
}

impl AnimationConfig {
    /// Validate animation configuration parameters
    pub fn validate(&self) -> Result<()> {
        // Validate planet_speed
        if self.planet_speed <= 0.0 || self.planet_speed > 5.0 {
            return Err(Error::Validation(
                format!("planet_speed must be between 0.0 and 5.0, got {}", self.planet_speed)
            ));
        }

        // Validate planet colors
        for (i, color) in self.planet_colors.iter().enumerate() {
            if color.len() != 4 {
                return Err(Error::Validation(format!("planet_colors[{}] must have 4 components", i)));
            }
            for (j, &component) in color.iter().enumerate() {
                if component < 0.0 || component > 1.0 {
                    return Err(Error::Validation(
                        format!("planet_colors[{}][{}] must be between 0.0 and 1.0, got {}", i, j, component)
                    ));
                }
            }
            if color[3] <= 0.0 {
                return Err(Error::Validation(
                    format!("planet_colors[{}] alpha must be > 0.0", i)
                ));
            }
        }

        // Validate planet sizes
        for (i, &size) in self.planet_sizes.iter().enumerate() {
            if size <= 0.0 {
                return Err(Error::Validation(
                    format!("planet_sizes[{}] must be > 0.0, got {}", i, size)
                ));
            }
        }

        // Validate sun color
        for (i, &component) in self.sun_color.iter().enumerate() {
            if component < 0.0 || component > 1.0 {
                return Err(Error::Validation(
                    format!("sun_color[{}] must be between 0.0 and 1.0, got {}", i, component)
                ));
            }
        }
        if self.sun_color[3] <= 0.0 {
            return Err(Error::Validation("sun_color alpha must be > 0.0".to_string()));
        }

        // Validate sun_size
        if self.sun_size <= 0.0 {
            return Err(Error::Validation(
                format!("sun_size must be > 0.0, got {}", self.sun_size)
            ));
        }

        Ok(())
    }
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            planet_speed: default_planet_speed(),
            planet_colors: default_planet_colors(),
            planet_sizes: default_planet_sizes(),
            sun_color: default_sun_color(),
            sun_size: default_sun_size(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_animation_config() {
        let config = AnimationConfig::default();
        assert_eq!(config.planet_speed, 1.0);
        assert_eq!(config.planet_colors.len(), 2);
        assert_eq!(config.planet_sizes.len(), 2);
    }

    #[test]
    fn test_animation_config_validation_passes() {
        let config = AnimationConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_planet_speed() {
        let mut config = AnimationConfig::default();
        config.planet_speed = 0.0;
        assert!(config.validate().is_err());

        config.planet_speed = 10.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_planet_color_component() {
        let mut config = AnimationConfig::default();
        config.planet_colors[0][0] = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_sun_color_alpha() {
        let mut config = AnimationConfig::default();
        config.sun_color[3] = 0.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_planet_size() {
        let mut config = AnimationConfig::default();
        config.planet_sizes[0] = 0.0;
        assert!(config.validate().is_err());
    }
}
