pub mod camera;
/// Renderer module for graphics rendering
pub mod scene;

use crate::error::Result;
use glam::UVec2;

/// wgpu render pipeline
pub struct RenderPipeline {
    pub viewport_size: UVec2,
    // GPU context would go here (Device, Queue, Surface, etc.)
    // Omitted for now to avoid complex wgpu setup
}

impl RenderPipeline {
    /// Create a new render pipeline
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let viewport_size = UVec2::new(width, height);

        if width == 0 || height == 0 {
            return Err(crate::error::Error::Render(
                "Viewport dimensions must be positive".to_string(),
            ));
        }

        Ok(RenderPipeline { viewport_size })
    }

    /// Initialize GPU context and render pass
    pub fn init(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Create wgpu instance
        // 2. Request adapter and device
        // 3. Create surface
        // 4. Create render pipeline
        Ok(())
    }

    /// Render a frame
    pub fn render(&self, _scene: &scene::Scene) -> Result<()> {
        // In a real implementation, this would:
        // 1. Create command encoder
        // 2. Create render pass
        // 3. Draw all scene entities
        // 4. Submit command buffer
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipeline_creation() {
        let pipeline = RenderPipeline::new(1920, 1080);
        assert!(pipeline.is_ok());
        let p = pipeline.unwrap();
        assert_eq!(p.viewport_size.x, 1920);
        assert_eq!(p.viewport_size.y, 1080);
    }

    #[test]
    fn test_render_pipeline_invalid_dimensions() {
        let pipeline = RenderPipeline::new(0, 1080);
        assert!(pipeline.is_err());
    }

    #[test]
    fn test_render_pipeline_init() {
        let pipeline = RenderPipeline::new(1920, 1080).unwrap();
        assert!(pipeline.init().is_ok());
    }
}
