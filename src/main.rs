/// Interactive OLED Wallpaper Application Entry Point
/// 
/// Handles window creation, render loop, and frame timing.

use oled_wallpaper::{init_tracing, error::Result, wallpaper, renderer, physics};
use glam::Vec3;
use tracing::info;
use std::time::Instant;

fn main() -> Result<()> {
    // Initialize logging
    init_tracing();
    info!("Application starting");

    // Detect display server
    let display_server = wallpaper::DisplayServer::detect()?;
    info!("Using {:?} display server", display_server);

    // Create wallpaper window
    let wallpaper_window = wallpaper::integration::WallpaperWindow::new(display_server, 1920, 1080);
    wallpaper_window.init()?;

    // Initialize renderer
    let renderer = renderer::RenderPipeline::new(1920, 1080)?;
    renderer.init()?;

    // Create scene with sun and 2 planets
    let mut scene = renderer::scene::Scene::new();
    
    // Add sun
    scene.add_entity(renderer::scene::SceneEntity::CelestialBody {
        id: "sun".to_string(),
        position: [0.0, 0.0, 0.0],
        radius: 10.0,
        color: [1.0, 0.9, 0.0, 1.0],
    });
    
    // Add planet 1
    scene.add_entity(renderer::scene::SceneEntity::CelestialBody {
        id: "planet_0".to_string(),
        position: [100.0, 0.0, 0.0],
        radius: 5.0,
        color: [0.2, 0.6, 1.0, 1.0],
    });
    
    // Add planet 2
    scene.add_entity(renderer::scene::SceneEntity::CelestialBody {
        id: "planet_1".to_string(),
        position: [0.0, 150.0, 0.0],
        radius: 4.0,
        color: [1.0, 0.4, 0.2, 1.0],
    });

    // Initialize physics simulator
    let mut simulator = physics::PhysicsSimulator::new();
    
    // Add sun to simulator
    let sun = physics::body::CelestialBody::sun("sun".to_string(), Vec3::ZERO);
    simulator.add_body(sun)?;
    
    // Add planets to simulator
    let mut planet1 = physics::body::CelestialBody::planet(
        "planet_0".to_string(),
        Vec3::new(100.0, 0.0, 0.0),
        5.0,
        glam::Vec4::new(0.2, 0.6, 1.0, 1.0),
    );
    planet1.velocity = Vec3::new(0.0, 62.8, 0.0); // Orbital velocity
    simulator.add_body(planet1)?;
    
    let mut planet2 = physics::body::CelestialBody::planet(
        "planet_1".to_string(),
        Vec3::new(0.0, 150.0, 0.0),
        4.0,
        glam::Vec4::new(1.0, 0.4, 0.2, 1.0),
    );
    planet2.velocity = Vec3::new(-51.1, 0.0, 0.0); // Orbital velocity
    simulator.add_body(planet2)?;
    
    // Add orbits
    let orbit1 = physics::orbit::Orbit::circular("planet_0".to_string(), "sun".to_string(), 100.0, 10.0);
    simulator.add_orbit(orbit1)?;
    
    let orbit2 = physics::orbit::Orbit::circular("planet_1".to_string(), "sun".to_string(), 150.0, 15.0);
    simulator.add_orbit(orbit2)?;

    // Main render loop
    let target_fps = 60.0f32;
    let frame_time_target = 1.0 / target_fps;
    let max_frames = 3600; // Run for up to 60 seconds at 60 FPS
    let start_time = Instant::now();
    
    info!("Starting render loop (target {} FPS)", target_fps);
    
    for frame_num in 0..max_frames {
        let frame_start = Instant::now();
        let elapsed_time = start_time.elapsed().as_secs_f32();
        
        // Update physics
        let delta_time = frame_time_target;
        simulator.update_all_bodies(delta_time, elapsed_time);
        
        // Update scene with new positions
        if let Some(body) = simulator.get_body("planet_0") {
            scene.remove_entity("planet_0");
            scene.add_entity(renderer::scene::SceneEntity::CelestialBody {
                id: "planet_0".to_string(),
                position: [body.position.x, body.position.y, body.position.z],
                radius: 5.0,
                color: [0.2, 0.6, 1.0, 1.0],
            });
        }
        
        if let Some(body) = simulator.get_body("planet_1") {
            scene.remove_entity("planet_1");
            scene.add_entity(renderer::scene::SceneEntity::CelestialBody {
                id: "planet_1".to_string(),
                position: [body.position.x, body.position.y, body.position.z],
                radius: 4.0,
                color: [1.0, 0.4, 0.2, 1.0],
            });
        }
        
        // Render frame
        renderer.render(&scene)?;
        
        // Frame timing
        let frame_elapsed = frame_start.elapsed().as_secs_f32();
        let sleep_time = frame_time_target - frame_elapsed;
        
        if sleep_time > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time));
        }
        
        // Print status every 300 frames (5 seconds at 60 FPS)
        if frame_num % 300 == 0 && frame_num > 0 {
            let total_elapsed = start_time.elapsed().as_secs_f32();
            let actual_fps = frame_num as f32 / total_elapsed;
            info!("Frame {}: {:.1} FPS", frame_num, actual_fps);
        }
        
        // Break early if running for a minute (simple demo)
        if elapsed_time > 60.0 {
            break;
        }
    }
    
    info!("Application shutting down gracefully");
    Ok(())
}

