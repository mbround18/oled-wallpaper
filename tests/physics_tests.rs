#![allow(clippy::empty_line_after_doc_comments)]

/// Unit tests for physics engine functionality
///
/// Tests for Kepler orbit calculations, celestial body updates, and validation.

#[cfg(test)]
mod physics_unit_tests {
    use glam::{Vec3, Vec4};
    use oled_wallpaper::physics::body::CelestialBody;
    use oled_wallpaper::physics::orbit::Orbit;

    /// Test: Kepler orbit position calculation at known times
    #[test]
    fn test_kepler_orbit_position_at_epoch() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        // At time 0, position should be at periapsis
        let pos = orbit.get_position_at_time(0.0);

        // For a circular orbit, distance should be constant at semi_major_axis
        let distance = pos.length();
        assert!(
            (distance - 100.0).abs() < 2.0,
            "Distance at epoch should be ~100"
        );
    }

    /// Test: Kepler orbit position changes over time
    #[test]
    fn test_kepler_orbit_position_changes_over_time() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        let pos_t0 = orbit.get_position_at_time(0.0);
        let pos_t1 = orbit.get_position_at_time(10.0);
        let pos_t2 = orbit.get_position_at_time(20.0);

        // Positions should be different
        assert!(pos_t0 != pos_t1, "Position should change over time");
        assert!(pos_t1 != pos_t2, "Position should continue changing");

        // Distances should remain constant for circular orbit
        let dist_t0 = pos_t0.length();
        let dist_t1 = pos_t1.length();
        let dist_t2 = pos_t2.length();

        assert!(
            (dist_t0 - dist_t1).abs() < 0.5,
            "Distance should remain constant for circular orbit"
        );
        assert!(
            (dist_t1 - dist_t2).abs() < 0.5,
            "Distance should remain constant for circular orbit"
        );
    }

    /// Test: Kepler orbit position after one complete orbit
    #[test]
    fn test_kepler_orbit_period() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        let pos_start = orbit.get_position_at_time(0.0);
        let pos_after_period = orbit.get_position_at_time(60.0);

        // After one period, position should return to start
        // Allow for numerical errors in the calculation
        assert!(
            (pos_start.x - pos_after_period.x).abs() < 0.5,
            "X position should return after period"
        );
        assert!(
            (pos_start.y - pos_after_period.y).abs() < 0.5,
            "Y position should return after period"
        );
    }

    /// Test: Orbital velocity calculation
    #[test]
    fn test_orbital_velocity_non_zero() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        let vel = orbit.get_velocity_at_time(0.0);

        // Velocity should be non-zero for a circular orbit
        let speed = vel.length();
        assert!(speed > 0.1, "Orbital velocity should be non-zero");
    }

    /// Test: Orbital velocity is tangent to position (perpendicular for circular orbit)
    #[test]
    fn test_orbital_velocity_tangent() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        let pos = orbit.get_position_at_time(10.0);
        let vel = orbit.get_velocity_at_time(10.0);

        // For a circular orbit, velocity should be approximately perpendicular to position
        let dot_product = pos.dot(vel);

        // Dot product of perpendicular vectors is ~0
        assert!(
            dot_product.abs() < 10.0,
            "Velocity should be approximately tangent to orbit"
        );
    }

    /// Test: Celestial body position update
    #[test]
    fn test_celestial_body_update_position() {
        let mut body = CelestialBody::planet("planet".to_string(), Vec3::ZERO, 5.0, Vec4::ONE);

        body.velocity = Vec3::new(10.0, 0.0, 0.0);

        body.update_position(1.0);
        assert_eq!(body.position.x, 10.0);

        body.update_position(1.0);
        assert_eq!(body.position.x, 20.0);
    }

    /// Test: Static bodies don't move
    #[test]
    fn test_static_body_no_movement() {
        let mut sun = CelestialBody::sun("sun".to_string(), Vec3::new(50.0, 50.0, 0.0));
        let initial_pos = sun.position;

        sun.velocity = Vec3::new(1000.0, 1000.0, 0.0);
        sun.update_position(10.0);

        assert_eq!(sun.position, initial_pos, "Static bodies should not move");
    }

    /// Test: Multiple updates accumulate
    #[test]
    fn test_multiple_position_updates() {
        let mut body = CelestialBody::planet("planet".to_string(), Vec3::ZERO, 5.0, Vec4::ONE);

        body.velocity = Vec3::new(5.0, 5.0, 0.0);

        // Update 10 times with delta_time = 1.0
        for _ in 0..10 {
            body.update_position(1.0);
        }

        // After 10 updates of 1.0s each with velocity (5, 5), position should be (50, 50)
        assert_eq!(body.position.x, 50.0);
        assert_eq!(body.position.y, 50.0);
    }

    /// Test: CelestialBody validation with edge cases
    #[test]
    fn test_validation_boundary_values() {
        // Minimum valid values
        let body_min = CelestialBody::new(
            "min".to_string(),
            Vec3::ZERO,
            0.001,                           // Very small but positive
            Vec4::new(0.0, 0.0, 0.0, 0.001), // Minimum alpha
            0.001,                           // Very small but positive mass
            false,
        );
        assert!(
            body_min.validate().is_ok(),
            "Should accept minimum valid values"
        );

        // Maximum valid values
        let body_max = CelestialBody::new(
            "max".to_string(),
            Vec3::new(10000.0, 10000.0, 10000.0),
            10000.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
            10000.0,
            true,
        );
        assert!(
            body_max.validate().is_ok(),
            "Should accept maximum valid values"
        );
    }

    /// Test: Orbit validation with edge cases
    #[test]
    fn test_orbit_validation_boundary_values() {
        // Minimum valid circular orbit
        let orbit_min = Orbit::circular(
            "planet".to_string(),
            "sun".to_string(),
            0.001, // Very small semi-major axis
            0.001, // Very short period
        );
        assert!(
            orbit_min.validate().is_ok(),
            "Should accept minimum valid orbit values"
        );

        // Maximum eccentricity (just under 1.0)
        let mut orbit_max_ecc =
            Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        orbit_max_ecc.eccentricity = 0.99;
        assert!(
            orbit_max_ecc.validate().is_ok(),
            "Should accept maximum eccentricity"
        );

        // Eccentricity at boundary (should fail)
        let mut orbit_ecc_1 = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        orbit_ecc_1.eccentricity = 1.0;
        assert!(
            orbit_ecc_1.validate().is_err(),
            "Should reject eccentricity = 1.0"
        );
    }

    /// Test: Validation catches invalid combinations
    #[test]
    fn test_validation_catches_invalid_combinations() {
        // Body with zero radius should fail
        let mut body =
            CelestialBody::new("test".to_string(), Vec3::ZERO, 1.0, Vec4::ONE, 1.0, false);
        body.radius = 0.0;
        assert!(body.validate().is_err(), "Should catch zero radius");

        // Orbit with body == parent should fail
        let mut orbit = Orbit::circular("body".to_string(), "sun".to_string(), 100.0, 60.0);
        orbit.parent_id = "body".to_string();
        assert!(orbit.validate().is_err(), "Should catch self-orbiting body");
    }
}
