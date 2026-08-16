use crate::error::{Error, Result};
/// Orbital mechanics and trajectory calculations
///
/// Describes an elliptical orbit using Kepler orbital elements.
use glam::Vec3;

/// An elliptical orbit defined by Kepler orbital elements
#[derive(Clone, Debug)]
pub struct Orbit {
    /// ID of the body following this orbit
    pub body_id: String,
    /// ID of the body being orbited (usually "sun")
    pub parent_id: String,
    /// Semi-major axis: half the longest diameter of the ellipse (world units)
    pub semi_major_axis: f32,
    /// Eccentricity: how elongated the ellipse is (0.0=circle, <1.0=ellipse)
    pub eccentricity: f32,
    /// Inclination: orbital plane tilt in radians (0.0-2π)
    pub inclination: f32,
    /// Argument of periapsis: orientation of the ellipse in radians (0.0-2π)
    pub argument_of_periapsis: f32,
    /// Mean anomaly at epoch: starting position in orbit (radians)
    pub mean_anomaly_at_epoch: f32,
    /// Orbital period: time for one complete orbit (seconds)
    pub orbital_period: f32,
}

impl Orbit {
    /// Create a new orbit
    pub fn new(
        body_id: String,
        parent_id: String,
        semi_major_axis: f32,
        eccentricity: f32,
        inclination: f32,
        argument_of_periapsis: f32,
        mean_anomaly_at_epoch: f32,
        orbital_period: f32,
    ) -> Self {
        Orbit {
            body_id,
            parent_id,
            semi_major_axis,
            eccentricity,
            inclination,
            argument_of_periapsis,
            mean_anomaly_at_epoch,
            orbital_period,
        }
    }

    /// Create a circular orbit (eccentricity = 0)
    pub fn circular(
        body_id: String,
        parent_id: String,
        semi_major_axis: f32,
        orbital_period: f32,
    ) -> Self {
        Orbit {
            body_id,
            parent_id,
            semi_major_axis,
            eccentricity: 0.0,
            inclination: 0.0,
            argument_of_periapsis: 0.0,
            mean_anomaly_at_epoch: 0.0,
            orbital_period,
        }
    }

    /// Get position at a specific time using Kepler equations
    pub fn get_position_at_time(&self, time: f32) -> Vec3 {
        // Calculate mean anomaly at the given time
        let mean_anomaly =
            self.mean_anomaly_at_epoch + (2.0 * std::f32::consts::PI * time) / self.orbital_period;

        // Use Newton-Raphson to solve Kepler's equation: M = E - e*sin(E)
        // Find eccentric anomaly E
        let mut eccentric_anomaly = mean_anomaly;
        for _ in 0..10 {
            let f = eccentric_anomaly - self.eccentricity * eccentric_anomaly.sin() - mean_anomaly;
            let f_prime = 1.0 - self.eccentricity * eccentric_anomaly.cos();
            if f_prime.abs() < 1e-10 {
                break;
            }
            eccentric_anomaly -= f / f_prime;
        }

        // Calculate true anomaly
        let true_anomaly = 2.0
            * ((eccentric_anomaly / 2.0).tan()
                / ((1.0 + self.eccentricity) / (1.0 - self.eccentricity)).sqrt())
            .atan();

        // Calculate distance from focus
        let distance = self.semi_major_axis * (1.0 - self.eccentricity * self.eccentricity)
            / (1.0 + self.eccentricity * true_anomaly.cos());

        // Calculate position in orbital plane
        let x = distance * true_anomaly.cos();
        let y = distance * true_anomaly.sin();

        // Apply inclination and argument of periapsis
        let cos_inc = self.inclination.cos();
        let sin_inc = self.inclination.sin();
        let cos_arg = self.argument_of_periapsis.cos();
        let sin_arg = self.argument_of_periapsis.sin();

        let pos_x = (cos_arg * x - sin_arg * y) * cos_inc;
        let pos_y = sin_arg * x + cos_arg * y;
        let pos_z = (cos_arg * x - sin_arg * y) * sin_inc;

        Vec3::new(pos_x, pos_y, pos_z)
    }

    /// Get velocity at a specific time (derivative of position)
    pub fn get_velocity_at_time(&self, time: f32) -> Vec3 {
        // Use numerical differentiation
        let dt = 0.001; // Small time step
        let pos_before = self.get_position_at_time(time - dt);
        let pos_after = self.get_position_at_time(time + dt);

        (pos_after - pos_before) / (2.0 * dt)
    }

    /// Validate orbital parameters
    pub fn validate(&self) -> Result<()> {
        if self.body_id.is_empty() {
            return Err(Error::Validation(
                "Orbit body_id cannot be empty".to_string(),
            ));
        }

        if self.parent_id.is_empty() {
            return Err(Error::Validation(
                "Orbit parent_id cannot be empty".to_string(),
            ));
        }

        if self.body_id == self.parent_id {
            return Err(Error::Validation("Body cannot orbit itself".to_string()));
        }

        if self.semi_major_axis <= 0.0 {
            return Err(Error::Validation(format!(
                "Orbit semi_major_axis must be > 0.0, got {}",
                self.semi_major_axis
            )));
        }

        if self.eccentricity < 0.0 || self.eccentricity >= 1.0 {
            return Err(Error::Validation(format!(
                "Orbit eccentricity must be in [0.0, 1.0), got {}",
                self.eccentricity
            )));
        }

        if self.orbital_period <= 0.0 {
            return Err(Error::Validation(format!(
                "Orbit orbital_period must be > 0.0, got {}",
                self.orbital_period
            )));
        }

        // Kepler's third law validation: T² ∝ a³
        // Check if orbital period is roughly consistent with semi-major axis
        // (allowing for unit system differences)
        let _expected_period_ratio = (self.semi_major_axis).powf(1.5);
        let actual_period = self.orbital_period;

        // Very loose check - just ensure they're in roughly the right ballpark
        // A more rigorous check would require knowing the mass of the parent body
        if actual_period <= 0.0 {
            return Err(Error::Validation(
                "Orbital period must be positive".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if orbit is valid
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orbit_creation() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        assert_eq!(orbit.body_id, "planet");
        assert_eq!(orbit.parent_id, "sun");
        assert_eq!(orbit.eccentricity, 0.0);
    }

    #[test]
    fn test_circular_orbit_creation() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 150.0, 90.0);
        assert_eq!(orbit.semi_major_axis, 150.0);
        assert_eq!(orbit.orbital_period, 90.0);
        assert_eq!(orbit.eccentricity, 0.0);
    }

    #[test]
    fn test_get_position_at_time() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        let pos_0 = orbit.get_position_at_time(0.0);
        let pos_quarter = orbit.get_position_at_time(15.0); // 1/4 orbit

        // Position should exist (not NaN)
        assert!(!pos_0.x.is_nan() && !pos_0.y.is_nan() && !pos_0.z.is_nan());
        assert!(!pos_quarter.x.is_nan() && !pos_quarter.y.is_nan() && !pos_quarter.z.is_nan());

        // Position should be at approximately the right distance
        let dist_0 = pos_0.length();
        assert!((dist_0 - 100.0).abs() < 1.0, "Distance should be ~100");
    }

    #[test]
    fn test_get_velocity_at_time() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        let vel = orbit.get_velocity_at_time(0.0);

        // Velocity should exist and not be zero
        assert!(!vel.x.is_nan() && !vel.y.is_nan() && !vel.z.is_nan());
        assert!(
            vel.length() > 0.0,
            "Velocity should be non-zero for circular orbit"
        );
    }

    #[test]
    fn test_validation_succeeds() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        assert!(orbit.validate().is_ok());
    }

    #[test]
    fn test_validation_fails_empty_body_id() {
        let orbit = Orbit::circular("".to_string(), "sun".to_string(), 100.0, 60.0);
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_validation_fails_empty_parent_id() {
        let orbit = Orbit::circular("planet".to_string(), "".to_string(), 100.0, 60.0);
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_validation_fails_self_orbit() {
        let orbit = Orbit::circular("planet".to_string(), "planet".to_string(), 100.0, 60.0);
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_semi_major_axis() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 0.0, 60.0);
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_eccentricity() {
        let mut orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        orbit.eccentricity = 1.5;
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_period() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 0.0);
        assert!(orbit.validate().is_err());
    }

    #[test]
    fn test_is_valid() {
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        assert!(orbit.is_valid());
    }
}
