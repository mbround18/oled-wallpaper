/// Celestial body physics and state management
///
/// Represents a single celestial object (sun or planet) in the galaxy scene.

use glam::Vec3;
use crate::error::{Error, Result};

/// A celestial body (sun or planet)
#[derive(Clone, Debug)]
pub struct CelestialBody {
    /// Unique identifier (e.g., "sun", "planet_0")
    pub id: String,
    /// Current 3D position in world space
    pub position: Vec3,
    /// Current velocity vector
    pub velocity: Vec3,
    /// Visual size/radius in pixels
    pub radius: f32,
    /// RGBA color (values 0.0-1.0)
    pub color: Vec4,
    /// Physical mass for orbit calculations
    pub mass: f32,
    /// Whether this body is static (sun) or dynamic (planets)
    pub is_static: bool,
}

use glam::Vec4;

impl CelestialBody {
    /// Create a new celestial body
    pub fn new(
        id: String,
        position: Vec3,
        radius: f32,
        color: Vec4,
        mass: f32,
        is_static: bool,
    ) -> Self {
        CelestialBody {
            id,
            position,
            velocity: Vec3::ZERO,
            radius,
            color,
            mass,
            is_static,
        }
    }

    /// Create a new sun body
    pub fn sun(id: String, position: Vec3) -> Self {
        CelestialBody {
            id,
            position,
            velocity: Vec3::ZERO,
            radius: 10.0,
            color: Vec4::new(1.0, 0.9, 0.0, 1.0),
            mass: 1000.0,
            is_static: true,
        }
    }

    /// Create a new planet body
    pub fn planet(id: String, position: Vec3, radius: f32, color: Vec4) -> Self {
        CelestialBody {
            id,
            position,
            velocity: Vec3::ZERO,
            radius,
            color,
            mass: 10.0,
            is_static: false,
        }
    }

    /// Update position based on velocity and elapsed time
    pub fn update_position(&mut self, delta_time: f32) {
        if !self.is_static {
            self.position += self.velocity * delta_time;
        }
    }

    /// Get screen coordinates (placeholder for camera transformation)
    pub fn get_screen_coordinates(&self) -> [f32; 2] {
        [self.position.x, self.position.y]
    }

    /// Check if a point intersects this body
    pub fn intersects_point(&self, point: [f32; 2], screen_radius: f32) -> bool {
        let dx = self.position.x - point[0];
        let dy = self.position.y - point[1];
        let distance = (dx * dx + dy * dy).sqrt();
        distance <= screen_radius + self.radius
    }

    /// Validate the celestial body
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::Validation("CelestialBody ID cannot be empty".to_string()));
        }

        if self.radius <= 0.0 {
            return Err(Error::Validation(
                format!("CelestialBody radius must be > 0.0, got {}", self.radius)
            ));
        }

        if self.mass <= 0.0 {
            return Err(Error::Validation(
                format!("CelestialBody mass must be > 0.0, got {}", self.mass)
            ));
        }

        if self.color.w <= 0.0 {
            return Err(Error::Validation(
                "CelestialBody color alpha must be > 0.0".to_string()
            ));
        }

        // Ensure all color components are in valid range
        if self.color.x < 0.0 || self.color.x > 1.0 ||
           self.color.y < 0.0 || self.color.y > 1.0 ||
           self.color.z < 0.0 || self.color.z > 1.0 ||
           self.color.w < 0.0 || self.color.w > 1.0 {
            return Err(Error::Validation(
                "CelestialBody color components must be in range [0.0, 1.0]".to_string()
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celestial_body_creation() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            1.0,
            false,
        );
        assert_eq!(body.id, "test");
        assert_eq!(body.radius, 5.0);
    }

    #[test]
    fn test_sun_creation() {
        let sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);
        assert!(sun.is_static);
        assert_eq!(sun.id, "sun");
    }

    #[test]
    fn test_planet_creation() {
        let planet = CelestialBody::planet(
            "planet_0".to_string(),
            Vec3::new(100.0, 0.0, 0.0),
            5.0,
            Vec4::new(0.2, 0.6, 1.0, 1.0),
        );
        assert!(!planet.is_static);
        assert_eq!(planet.id, "planet_0");
    }

    #[test]
    fn test_update_position() {
        let mut body = CelestialBody::planet(
            "planet".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::ONE,
        );
        body.velocity = Vec3::new(1.0, 2.0, 0.0);
        
        body.update_position(1.0);
        
        assert_eq!(body.position.x, 1.0);
        assert_eq!(body.position.y, 2.0);
    }

    #[test]
    fn test_static_body_doesnt_move() {
        let mut sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);
        sun.velocity = Vec3::new(10.0, 10.0, 0.0);
        
        sun.update_position(1.0);
        
        assert_eq!(sun.position, Vec3::ZERO);
    }

    #[test]
    fn test_validation_succeeds() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::new(1.0, 0.0, 0.0, 1.0),
            1.0,
            false,
        );
        assert!(body.validate().is_ok());
    }

    #[test]
    fn test_validation_fails_empty_id() {
        let body = CelestialBody::new(
            "".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::ONE,
            1.0,
            false,
        );
        assert!(body.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_radius() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::ZERO,
            0.0,
            Vec4::ONE,
            1.0,
            false,
        );
        assert!(body.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_mass() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::ONE,
            0.0,
            false,
        );
        assert!(body.validate().is_err());
    }

    #[test]
    fn test_validation_fails_invalid_alpha() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::ZERO,
            5.0,
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            1.0,
            false,
        );
        assert!(body.validate().is_err());
    }

    #[test]
    fn test_intersects_point() {
        let body = CelestialBody::new(
            "test".to_string(),
            Vec3::new(100.0, 100.0, 0.0),
            10.0,
            Vec4::ONE,
            1.0,
            false,
        );
        
        // Point at center
        assert!(body.intersects_point([100.0, 100.0], 0.0));
        
        // Point outside
        assert!(!body.intersects_point([200.0, 200.0], 0.0));
    }
}
