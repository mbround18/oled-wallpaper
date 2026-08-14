/// Camera/Viewport management for rendering
/// 
/// Handles viewport transformation and coordinate conversions between world and screen space.

use glam::{Vec2, Vec3};

/// Camera and viewport configuration
pub struct Camera {
    pub position: Vec2,
    pub zoom_level: f32,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    /// Create a new camera at the specified viewport dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Camera {
            position: Vec2::ZERO,
            zoom_level: 1.0,
            width,
            height,
        }
    }

    /// Pan the camera by a delta amount
    pub fn pan_by(&mut self, delta: Vec2) {
        self.position += delta;
    }

    /// Set the zoom level
    pub fn set_zoom(&mut self, level: f32) {
        const MIN_ZOOM: f32 = 0.1;
        const MAX_ZOOM: f32 = 10.0;
        self.zoom_level = level.clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// Convert screen coordinates to world space
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec3 {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        
        let offset_x = (screen_pos.x - center_x) / self.zoom_level;
        let offset_y = (screen_pos.y - center_y) / self.zoom_level;
        
        Vec3::new(
            self.position.x + offset_x,
            self.position.y + offset_y,
            0.0,
        )
    }

    /// Convert world coordinates to screen space
    pub fn world_to_screen(&self, world_pos: Vec3) -> Vec2 {
        let center_x = self.width as f32 / 2.0;
        let center_y = self.height as f32 / 2.0;
        
        let screen_x = center_x + (world_pos.x - self.position.x) * self.zoom_level;
        let screen_y = center_y + (world_pos.y - self.position.y) * self.zoom_level;
        
        Vec2::new(screen_x, screen_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new(1920, 1080);
        assert_eq!(camera.width, 1920);
        assert_eq!(camera.height, 1080);
        assert_eq!(camera.zoom_level, 1.0);
    }

    #[test]
    fn test_pan_camera() {
        let mut camera = Camera::new(1920, 1080);
        camera.pan_by(Vec2::new(10.0, 20.0));
        assert_eq!(camera.position.x, 10.0);
        assert_eq!(camera.position.y, 20.0);
    }

    #[test]
    fn test_set_zoom_clamps() {
        let mut camera = Camera::new(1920, 1080);
        
        camera.set_zoom(0.01);
        assert!(camera.zoom_level >= 0.1);
        
        camera.set_zoom(100.0);
        assert!(camera.zoom_level <= 10.0);
    }

    #[test]
    fn test_screen_to_world_at_center() {
        let camera = Camera::new(1920, 1080);
        let screen_pos = Vec2::new(960.0, 540.0); // Center of screen
        let world_pos = camera.screen_to_world(screen_pos);
        
        // At center of screen with camera at origin, should be at origin
        assert!(world_pos.x.abs() < 0.01);
        assert!(world_pos.y.abs() < 0.01);
    }

    #[test]
    fn test_world_to_screen_and_back() {
        let camera = Camera::new(1920, 1080);
        let world_pos = Vec3::new(100.0, 50.0, 0.0);
        
        let screen_pos = camera.world_to_screen(world_pos);
        let back_to_world = camera.screen_to_world(screen_pos);
        
        assert!((back_to_world.x - world_pos.x).abs() < 0.01);
        assert!((back_to_world.y - world_pos.y).abs() < 0.01);
    }
}
