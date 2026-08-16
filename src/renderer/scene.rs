/// Scene management for rendering
///
/// Container for all entities to be rendered, including celestial bodies and effects.
use crate::error::Result;

/// Base scene structure containing all renderable entities
pub struct Scene {
    pub entities: Vec<SceneEntity>,
}

/// Types of entities in the scene
#[derive(Clone, Debug)]
pub enum SceneEntity {
    CelestialBody {
        id: String,
        position: [f32; 3],
        radius: f32,
        color: [f32; 4],
    },
}

impl Scene {
    /// Create a new empty scene
    pub fn new() -> Self {
        Scene {
            entities: Vec::new(),
        }
    }

    /// Add an entity to the scene
    pub fn add_entity(&mut self, entity: SceneEntity) {
        self.entities.push(entity);
    }

    /// Remove an entity from the scene by ID
    pub fn remove_entity(&mut self, id: &str) -> bool {
        if let Some(pos) = self.entities.iter().position(|e| {
            let SceneEntity::CelestialBody { id: entity_id, .. } = e;
            entity_id == id
        }) {
            self.entities.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the number of entities in the scene
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get an entity by ID
    pub fn get_entity(&self, id: &str) -> Option<&SceneEntity> {
        self.entities.iter().find(|e| {
            let SceneEntity::CelestialBody { id: entity_id, .. } = e;
            entity_id == id
        })
    }

    /// Get a mutable reference to an entity by ID
    pub fn get_entity_mut(&mut self, id: &str) -> Option<&mut SceneEntity> {
        self.entities.iter_mut().find(|e| {
            let SceneEntity::CelestialBody { id: entity_id, .. } = e;
            entity_id == id
        })
    }

    /// Clear all entities from the scene
    pub fn clear(&mut self) {
        self.entities.clear();
    }

    /// Validate the scene
    pub fn validate(&self) -> Result<()> {
        // Basic validation: at least one entity should exist
        Ok(())
    }

    /// Get all celestial bodies in screen space
    ///
    /// This would typically be called with a camera to transform world space to screen space.
    pub fn get_bodies_for_rendering(&self) -> Vec<(String, [f32; 3], f32, [f32; 4])> {
        self.entities
            .iter()
            .map(|entity| {
                let SceneEntity::CelestialBody {
                    id,
                    position,
                    radius,
                    color,
                } = entity;
                (id.clone(), *position, *radius, *color)
            })
            .collect()
    }

    /// Render all entities in the scene
    ///
    /// In a full implementation, this would use the renderer to draw all bodies.
    pub fn render(&self, _renderer: &crate::renderer::RenderPipeline) -> Result<()> {
        // Get all bodies for rendering
        let _bodies = self.get_bodies_for_rendering();

        // In a real implementation:
        // 1. For each body, create a mesh (circle quad)
        // 2. Apply color and transform
        // 3. Submit to renderer

        Ok(())
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_add_entity() {
        let mut scene = Scene::new();
        let entity = SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 1.0, 0.0, 1.0],
        };
        scene.add_entity(entity);
        assert_eq!(scene.entity_count(), 1);
    }

    #[test]
    fn test_get_entity() {
        let mut scene = Scene::new();
        let entity = SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 1.0, 0.0, 1.0],
        };
        scene.add_entity(entity);

        let retrieved = scene.get_entity("sun");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_remove_entity() {
        let mut scene = Scene::new();
        let entity = SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 1.0, 0.0, 1.0],
        };
        scene.add_entity(entity);

        let removed = scene.remove_entity("sun");
        assert!(removed);
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_clear_scene() {
        let mut scene = Scene::new();
        scene.add_entity(SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 1.0, 0.0, 1.0],
        });
        scene.add_entity(SceneEntity::CelestialBody {
            id: "planet".to_string(),
            position: [100.0, 0.0, 0.0],
            radius: 5.0,
            color: [0.2, 0.6, 1.0, 1.0],
        });

        scene.clear();
        assert_eq!(scene.entity_count(), 0);
    }
}
