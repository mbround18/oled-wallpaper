// Physics simulation module
pub mod body;
pub mod orbit;

use crate::error::Result;
use body::CelestialBody;
use orbit::Orbit;

/// Physics simulation coordinator
pub struct PhysicsSimulator {
    pub bodies: Vec<CelestialBody>,
    pub orbits: Vec<Orbit>,
}

impl PhysicsSimulator {
    /// Create a new physics simulator
    pub fn new() -> Self {
        PhysicsSimulator {
            bodies: Vec::new(),
            orbits: Vec::new(),
        }
    }

    /// Add a body to the simulation
    pub fn add_body(&mut self, body: CelestialBody) -> Result<()> {
        body.validate()?;
        self.bodies.push(body);
        Ok(())
    }

    /// Add an orbit to the simulation
    pub fn add_orbit(&mut self, orbit: Orbit) -> Result<()> {
        orbit.validate()?;
        self.orbits.push(orbit);
        Ok(())
    }

    /// Update all bodies in the simulation
    pub fn update_all_bodies(&mut self, delta_time: f32, current_time: f32) {
        // Update each body's position based on its orbit
        for orbit in &self.orbits {
            if let Some(body) = self.bodies.iter_mut().find(|b| b.id == orbit.body_id) {
                let position = orbit.get_position_at_time(current_time);
                body.position = position;

                let velocity = orbit.get_velocity_at_time(current_time);
                body.velocity = velocity;
            }
        }

        // Update all dynamic bodies
        for body in &mut self.bodies {
            body.update_position(delta_time);
        }
    }

    /// Get a body by ID
    pub fn get_body(&self, id: &str) -> Option<&CelestialBody> {
        self.bodies.iter().find(|b| b.id == id)
    }

    /// Get an orbit by body ID
    pub fn get_orbit(&self, body_id: &str) -> Option<&Orbit> {
        self.orbits.iter().find(|o| o.body_id == body_id)
    }

    /// Validate the entire simulation
    pub fn validate(&self) -> Result<()> {
        // Validate all bodies
        for body in &self.bodies {
            body.validate()?;
        }

        // Validate all orbits
        for orbit in &self.orbits {
            orbit.validate()?;
        }

        // Check that there's exactly one static body (the sun)
        let static_count = self.bodies.iter().filter(|b| b.is_static).count();
        if static_count > 1 {
            return Err(crate::error::Error::Validation(format!(
                "Expected exactly 1 static body, found {}",
                static_count
            )));
        }

        Ok(())
    }
}

impl Default for PhysicsSimulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec3, Vec4};

    #[test]
    fn test_physics_simulator_creation() {
        let sim = PhysicsSimulator::new();
        assert_eq!(sim.bodies.len(), 0);
        assert_eq!(sim.orbits.len(), 0);
    }

    #[test]
    fn test_add_body() {
        let mut sim = PhysicsSimulator::new();
        let sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);

        assert!(sim.add_body(sun).is_ok());
        assert_eq!(sim.bodies.len(), 1);
    }

    #[test]
    fn test_add_invalid_body() {
        let mut sim = PhysicsSimulator::new();
        let mut body = CelestialBody::planet("planet".to_string(), Vec3::ZERO, 5.0, Vec4::ONE);
        body.radius = 0.0; // Invalid

        assert!(sim.add_body(body).is_err());
        assert_eq!(sim.bodies.len(), 0);
    }

    #[test]
    fn test_add_orbit() {
        let mut sim = PhysicsSimulator::new();
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);

        assert!(sim.add_orbit(orbit).is_ok());
        assert_eq!(sim.orbits.len(), 1);
    }

    #[test]
    fn test_update_all_bodies() {
        let mut sim = PhysicsSimulator::new();

        // Add sun
        let sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);
        sim.add_body(sun).unwrap();

        // Add planet
        let mut planet = CelestialBody::planet(
            "planet".to_string(),
            Vec3::new(100.0, 0.0, 0.0),
            5.0,
            Vec4::ONE,
        );
        planet.velocity = Vec3::new(0.0, 10.0, 0.0);
        sim.add_body(planet).unwrap();

        // Add orbit
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        sim.add_orbit(orbit).unwrap();

        let initial_pos = sim.get_body("planet").unwrap().position;

        // Update
        sim.update_all_bodies(1.0, 5.0);

        let new_pos = sim.get_body("planet").unwrap().position;

        // Position should change
        assert_ne!(initial_pos, new_pos);
    }

    #[test]
    fn test_validate_simulation() {
        let mut sim = PhysicsSimulator::new();

        // Add sun
        let sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);
        sim.add_body(sun).unwrap();

        // Add planet
        let planet = CelestialBody::planet(
            "planet".to_string(),
            Vec3::new(100.0, 0.0, 0.0),
            5.0,
            Vec4::ONE,
        );
        sim.add_body(planet).unwrap();

        // Add orbit
        let orbit = Orbit::circular("planet".to_string(), "sun".to_string(), 100.0, 60.0);
        sim.add_orbit(orbit).unwrap();

        assert!(sim.validate().is_ok());
    }

    #[test]
    fn test_validate_fails_multiple_suns() {
        let mut sim = PhysicsSimulator::new();

        // Add first sun
        let sun1 = CelestialBody::sun("sun1".to_string(), Vec3::ZERO);
        sim.add_body(sun1).unwrap();

        // Add second sun (invalid)
        let sun2 = CelestialBody::sun("sun2".to_string(), Vec3::new(100.0, 0.0, 0.0));
        sim.add_body(sun2).unwrap();

        // Validation should fail due to multiple static bodies
        assert!(sim.validate().is_err());
    }
}
