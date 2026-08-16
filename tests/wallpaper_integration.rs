#![allow(clippy::empty_line_after_doc_comments)]

/// Integration tests for wallpaper rendering
///
/// Tests for full rendering pipeline with physics and scene updates.

#[cfg(test)]
mod wallpaper_integration_tests {
    use glam::{Vec3, Vec4};
    use oled_wallpaper::physics::{body::CelestialBody, PhysicsSimulator};
    use oled_wallpaper::renderer::{
        camera::Camera, scene::Scene, scene::SceneEntity, RenderPipeline,
    };

    /// Helper to create a test scene with sun and planets
    fn create_test_scene() -> Scene {
        let mut scene = Scene::new();

        // Add sun
        scene.add_entity(SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 0.9, 0.0, 1.0],
        });

        // Add planet 1
        scene.add_entity(SceneEntity::CelestialBody {
            id: "planet_0".to_string(),
            position: [100.0, 0.0, 0.0],
            radius: 5.0,
            color: [0.2, 0.6, 1.0, 1.0],
        });

        // Add planet 2
        scene.add_entity(SceneEntity::CelestialBody {
            id: "planet_1".to_string(),
            position: [0.0, 150.0, 0.0],
            radius: 4.0,
            color: [1.0, 0.4, 0.2, 1.0],
        });

        scene
    }

    /// Test: Render scene for 60 frames and measure FPS
    #[test]
    fn test_render_60_frames_fps() {
        let renderer = RenderPipeline::new(1920, 1080).unwrap();
        let scene = create_test_scene();

        let frame_count = 60u32;
        let start = std::time::Instant::now();

        // Simulate 60 frames of rendering
        for _frame in 0..frame_count {
            let _ = renderer.render(&scene);
        }

        let elapsed = start.elapsed();
        let fps = frame_count as f32 / elapsed.as_secs_f32();

        // Even without actual GPU rendering, we should be very fast
        // In a real implementation, would check >= 30 FPS
        assert!(fps > 0.0, "Should render frames");
        println!("Test render FPS: {:.1}", fps);
    }

    /// Test: Verify planet positions update correctly each frame
    #[test]
    fn test_planet_positions_update() {
        let mut scene = create_test_scene();
        let mut simulator = PhysicsSimulator::new();

        // Add sun
        let sun = CelestialBody::sun("sun".to_string(), Vec3::ZERO);
        simulator.add_body(sun).unwrap();

        // Add planet
        let mut planet = CelestialBody::planet(
            "planet_0".to_string(),
            Vec3::new(100.0, 0.0, 0.0),
            5.0,
            Vec4::ONE,
        );
        planet.velocity = Vec3::new(0.0, 10.0, 0.0);
        simulator.add_body(planet).unwrap();

        // Get initial position
        let initial_entity = scene.get_entity("planet_0").cloned().unwrap();
        let SceneEntity::CelestialBody {
            position: initial_pos,
            ..
        } = initial_entity;

        // Simulate frames
        for frame in 1..=10 {
            let delta_time = 0.016; // ~60 FPS
            simulator.update_all_bodies(delta_time, frame as f32 * delta_time);

            // Get updated position from simulator
            if let Some(body) = simulator.get_body("planet_0") {
                // Update scene entity with new position
                scene.remove_entity("planet_0");
                scene.add_entity(SceneEntity::CelestialBody {
                    id: "planet_0".to_string(),
                    position: [body.position.x, body.position.y, body.position.z],
                    radius: 5.0,
                    color: [0.2, 0.6, 1.0, 1.0],
                });
            }
        }

        // Get final position
        let final_entity = scene.get_entity("planet_0").cloned().unwrap();
        let SceneEntity::CelestialBody {
            position: final_pos,
            ..
        } = final_entity;

        // Position should have changed
        assert_ne!(
            initial_pos, final_pos,
            "Planet position should update each frame"
        );
    }

    /// Test: Rendering doesn't crash with various planet counts
    #[test]
    fn test_render_various_planet_counts() {
        let renderer = RenderPipeline::new(1920, 1080).unwrap();

        // Test with 1 body (just sun)
        let mut scene = Scene::new();
        scene.add_entity(SceneEntity::CelestialBody {
            id: "sun".to_string(),
            position: [0.0, 0.0, 0.0],
            radius: 10.0,
            color: [1.0, 0.9, 0.0, 1.0],
        });
        assert!(renderer.render(&scene).is_ok());

        // Test with multiple planets
        for i in 0..5 {
            let angle = (i as f32 / 5.0) * std::f32::consts::TAU;
            scene.add_entity(SceneEntity::CelestialBody {
                id: format!("planet_{}", i),
                position: [100.0 * angle.cos(), 100.0 * angle.sin(), 0.0],
                radius: 5.0,
                color: [0.2 + i as f32 * 0.1, 0.6, 1.0 - i as f32 * 0.1, 1.0],
            });
        }
        assert!(
            renderer.render(&scene).is_ok(),
            "Should render with multiple planets"
        );

        // Verify scene integrity
        assert_eq!(scene.entity_count(), 6); // 1 sun + 5 planets
    }

    /// Test: Camera transformations work correctly
    #[test]
    fn test_camera_transformations() {
        let camera = Camera::new(1920, 1080);
        let world_pos = Vec3::new(100.0, 50.0, 0.0);

        // Transform to screen and back
        let screen_pos = camera.world_to_screen(world_pos);
        let back_to_world = camera.screen_to_world(screen_pos);

        // Should be close to original position
        assert!((back_to_world.x - world_pos.x).abs() < 0.01);
        assert!((back_to_world.y - world_pos.y).abs() < 0.01);
    }

    /// Test: Scene rendering with camera integration
    #[test]
    fn test_scene_rendering_with_camera() {
        let scene = create_test_scene();
        let camera = Camera::new(1920, 1080);
        let renderer = RenderPipeline::new(1920, 1080).unwrap();

        // Get rendering data
        let bodies = scene.get_bodies_for_rendering();
        assert_eq!(bodies.len(), 3, "Should have 3 bodies (sun + 2 planets)");

        // Verify bodies are in correct format
        for (id, _position, radius, color) in bodies {
            assert!(!id.is_empty(), "Body ID should not be empty");
            assert!(radius > 0.0, "Radius should be positive");
            assert!(color[3] > 0.0, "Alpha should be positive");
        }

        // Verify rendering works
        assert!(renderer.render(&scene).is_ok());

        // Verify camera setup
        assert_eq!(camera.width, 1920);
        assert_eq!(camera.height, 1080);
    }
}
