#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::field_reassign_with_default)]

/// Unit tests for configuration loading and validation

#[cfg(test)]
mod config_tests {
    use oled_wallpaper::config::{animation::AnimationConfig, Config};

    /// Test: Default configuration loads correctly
    #[test]
    fn test_default_config_loads() {
        let config = Config::default();
        assert!(config.validate().is_ok(), "Default config should be valid");
    }

    /// Test: Animation config has valid defaults
    #[test]
    fn test_animation_config_defaults_valid() {
        let config = AnimationConfig::default();
        assert!(
            config.validate().is_ok(),
            "Default animation config should be valid"
        );
        assert_eq!(config.planet_speed, 1.0);
        assert_eq!(config.planet_colors.len(), 2);
        assert_eq!(config.planet_sizes.len(), 2);
        assert_eq!(config.sun_size, 2.0);
    }

    /// Test: Valid configuration with custom values
    #[test]
    fn test_valid_custom_config() {
        let mut config = AnimationConfig::default();
        config.planet_speed = 2.5;
        config.sun_size = 5.0;

        assert!(
            config.validate().is_ok(),
            "Custom config with valid values should pass"
        );
    }

    /// Test: Invalid planet speed (too high) is caught
    #[test]
    fn test_invalid_planet_speed_too_high() {
        let mut config = AnimationConfig::default();
        config.planet_speed = 10.0; // Maximum is 5.0

        assert!(
            config.validate().is_err(),
            "Should reject planet_speed > 5.0"
        );
    }

    /// Test: Invalid planet speed (too low) is caught
    #[test]
    fn test_invalid_planet_speed_too_low() {
        let mut config = AnimationConfig::default();
        config.planet_speed = 0.0; // Minimum is > 0.0

        assert!(
            config.validate().is_err(),
            "Should reject planet_speed <= 0.0"
        );
    }

    /// Test: Invalid color component is caught
    #[test]
    fn test_invalid_color_component() {
        let mut config = AnimationConfig::default();
        config.sun_color[0] = 1.5; // Color component must be 0.0-1.0

        assert!(
            config.validate().is_err(),
            "Should reject color component > 1.0"
        );
    }

    /// Test: Invalid color component (negative) is caught
    #[test]
    fn test_invalid_color_component_negative() {
        let mut config = AnimationConfig::default();
        config.sun_color[1] = -0.5; // Color component must be 0.0-1.0

        assert!(
            config.validate().is_err(),
            "Should reject negative color component"
        );
    }

    /// Test: Invalid alpha (transparency) is caught
    #[test]
    fn test_invalid_alpha_zero() {
        let mut config = AnimationConfig::default();
        config.sun_color[3] = 0.0; // Alpha must be > 0.0

        assert!(
            config.validate().is_err(),
            "Should reject alpha = 0.0 (fully transparent)"
        );
    }

    /// Test: Out-of-range planet speed is clamped to valid range
    #[test]
    fn test_speed_clamping_would_work() {
        let mut config = AnimationConfig::default();

        // Test boundary values
        config.planet_speed = 0.1; // Minimum valid
        assert!(config.validate().is_ok(), "Should accept speed = 0.1");

        config.planet_speed = 5.0; // Maximum valid
        assert!(config.validate().is_ok(), "Should accept speed = 5.0");
    }

    /// Test: Color component boundary values
    #[test]
    fn test_color_boundary_values() {
        let mut config = AnimationConfig::default();

        // Set to minimum valid
        config.sun_color = [0.0, 0.0, 0.0, 0.001];
        assert!(
            config.validate().is_ok(),
            "Should accept minimum color values"
        );

        // Set to maximum valid
        config.sun_color = [1.0, 1.0, 1.0, 1.0];
        assert!(
            config.validate().is_ok(),
            "Should accept maximum color values"
        );
    }

    /// Test: Multiple color components out of range
    #[test]
    fn test_multiple_invalid_colors() {
        let mut config = AnimationConfig::default();
        config.planet_colors[0][0] = 1.5; // R invalid
        config.planet_colors[0][1] = -0.2; // G invalid

        assert!(
            config.validate().is_err(),
            "Should reject multiple invalid colors"
        );
    }

    /// Test: Planet size validation
    #[test]
    fn test_planet_size_validation() {
        let mut config = AnimationConfig::default();

        // Test valid size
        config.planet_sizes[0] = 0.1;
        assert!(
            config.validate().is_ok(),
            "Should accept positive planet size"
        );

        // Test invalid size (zero)
        config.planet_sizes[0] = 0.0;
        assert!(config.validate().is_err(), "Should reject zero planet size");

        // Test invalid size (negative)
        config.planet_sizes[0] = -1.0;
        assert!(
            config.validate().is_err(),
            "Should reject negative planet size"
        );
    }

    /// Test: Sun size validation
    #[test]
    fn test_sun_size_validation() {
        let mut config = AnimationConfig::default();

        // Test valid size
        config.sun_size = 10.0;
        assert!(config.validate().is_ok(), "Should accept valid sun size");

        // Test invalid size
        config.sun_size = 0.0;
        assert!(config.validate().is_err(), "Should reject zero sun size");
    }

    /// Test: Configuration validation summary
    #[test]
    fn test_full_config_validation_chain() {
        // Create a complex valid configuration
        let mut config = AnimationConfig::default();
        config.planet_speed = 3.5;
        config.planet_colors = vec![
            [1.0, 0.0, 0.0, 1.0], // Red
            [0.0, 1.0, 0.0, 1.0], // Green
            [0.0, 0.0, 1.0, 1.0], // Blue
        ];
        config.planet_sizes = vec![0.5, 0.75, 1.0];
        config.sun_color = [1.0, 1.0, 0.0, 1.0]; // Yellow
        config.sun_size = 3.0;

        // Should pass all validation
        assert!(
            config.validate().is_ok(),
            "Complex valid config should pass"
        );
    }
}
