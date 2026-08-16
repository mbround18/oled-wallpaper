#![allow(clippy::empty_line_after_doc_comments)]

/// Common test infrastructure and helper functions
///
/// Provides mock objects and utility functions for testing.

#[allow(dead_code)]
use glam::{Vec2, Vec3};

/// Mock celestial body for testing
pub struct MockCelestialBody {
    pub id: String,
    pub position: Vec3,
    pub velocity: Vec3,
    pub radius: f32,
    pub color: [f32; 4],
    pub mass: f32,
    pub is_static: bool,
}

impl MockCelestialBody {
    /// Create a new mock celestial body
    pub fn new(id: &str) -> Self {
        MockCelestialBody {
            id: id.to_string(),
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            radius: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            mass: 1.0,
            is_static: false,
        }
    }

    /// Create a mock sun body
    pub fn sun() -> Self {
        let mut body = Self::new("sun");
        body.is_static = true;
        body.radius = 10.0;
        body.color = [1.0, 0.9, 0.0, 1.0];
        body.mass = 1000.0;
        body
    }

    /// Create a mock planet body
    pub fn planet(id: &str) -> Self {
        let mut body = Self::new(id);
        body.radius = 5.0;
        body.color = [0.2, 0.6, 1.0, 1.0];
        body.mass = 10.0;
        body
    }
}

/// Mock camera for testing
pub struct MockCamera {
    pub position: Vec2,
    pub zoom_level: f32,
    pub width: u32,
    pub height: u32,
}

impl MockCamera {
    /// Create a new mock camera
    pub fn new(width: u32, height: u32) -> Self {
        MockCamera {
            position: Vec2::ZERO,
            zoom_level: 1.0,
            width,
            height,
        }
    }
}

/// Mock orbit for testing
pub struct MockOrbit {
    pub body_id: String,
    pub parent_id: String,
    pub semi_major_axis: f32,
    pub eccentricity: f32,
    pub inclination: f32,
    pub orbital_period: f32,
    pub mean_anomaly_at_epoch: f32,
}

impl MockOrbit {
    /// Create a new mock orbit
    pub fn new(body_id: &str, parent_id: &str) -> Self {
        MockOrbit {
            body_id: body_id.to_string(),
            parent_id: parent_id.to_string(),
            semi_major_axis: 100.0,
            eccentricity: 0.2,
            inclination: 0.0,
            orbital_period: 60.0,
            mean_anomaly_at_epoch: 0.0,
        }
    }

    /// Create a valid circular orbit
    pub fn circular(body_id: &str) -> Self {
        let mut orbit = Self::new(body_id, "sun");
        orbit.eccentricity = 0.0; // Circular orbit
        orbit
    }

    /// Create an elliptical orbit
    pub fn elliptical(body_id: &str) -> Self {
        let mut orbit = Self::new(body_id, "sun");
        orbit.eccentricity = 0.5; // Elliptical orbit
        orbit
    }
}

/// Test helper: create a simple test scene
pub fn create_test_scene() -> (Vec<MockCelestialBody>, Vec<MockOrbit>) {
    let sun = MockCelestialBody::sun();
    let planet1 = {
        let mut p = MockCelestialBody::planet("planet_0");
        p.position = Vec3::new(100.0, 0.0, 0.0);
        p
    };
    let planet2 = {
        let mut p = MockCelestialBody::planet("planet_1");
        p.position = Vec3::new(-150.0, 0.0, 0.0);
        p
    };

    let orbit1 = MockOrbit::circular("planet_0");
    let orbit2 = MockOrbit::elliptical("planet_1");

    (vec![sun, planet1, planet2], vec![orbit1, orbit2])
}

/// Test helper: approximate float equality
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// Test helper: approximate vector equality
pub fn approx_vec_eq(a: Vec3, b: Vec3, epsilon: f32) -> bool {
    approx_eq(a.x, b.x, epsilon) && approx_eq(a.y, b.y, epsilon) && approx_eq(a.z, b.z, epsilon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_celestial_body_creation() {
        let body = MockCelestialBody::new("test");
        assert_eq!(body.id, "test");
        assert_eq!(body.radius, 1.0);
    }

    #[test]
    fn test_mock_sun() {
        let sun = MockCelestialBody::sun();
        assert!(sun.is_static);
        assert_eq!(sun.id, "sun");
    }

    #[test]
    fn test_mock_planet() {
        let planet = MockCelestialBody::planet("planet_0");
        assert!(!planet.is_static);
        assert_eq!(planet.id, "planet_0");
    }

    #[test]
    fn test_mock_camera() {
        let camera = MockCamera::new(1920, 1080);
        assert_eq!(camera.width, 1920);
        assert_eq!(camera.height, 1080);
    }

    #[test]
    fn test_create_test_scene() {
        let (bodies, orbits) = create_test_scene();
        assert_eq!(bodies.len(), 3); // sun + 2 planets
        assert_eq!(orbits.len(), 2); // 2 planet orbits
    }

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0001, 0.001));
        assert!(!approx_eq(1.0, 2.0, 0.5));
    }
}
