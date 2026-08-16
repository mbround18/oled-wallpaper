/// Wallpaper integration and display server management
///
/// Handles X11 and Wayland display server detection and initialization.
pub mod integration;

use crate::error::Result;
use tracing::{info, warn};

/// Display server type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    X11,
    Wayland,
}

impl DisplayServer {
    /// Detect the current display server
    pub fn detect() -> Result<DisplayServer> {
        // First, check environment variables
        if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
            match session_type.as_str() {
                "x11" => {
                    info!("Detected X11 display server (XDG_SESSION_TYPE)");
                    return Ok(DisplayServer::X11);
                }
                "wayland" => {
                    info!("Detected Wayland display server (XDG_SESSION_TYPE)");
                    return Ok(DisplayServer::Wayland);
                }
                _ => {}
            }
        }

        // Check for WAYLAND_DISPLAY environment variable
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            info!("Detected Wayland display server (WAYLAND_DISPLAY set)");
            return Ok(DisplayServer::Wayland);
        }

        // Check for DISPLAY environment variable (X11)
        if std::env::var("DISPLAY").is_ok() {
            info!("Detected X11 display server (DISPLAY set)");
            return Ok(DisplayServer::X11);
        }

        // Fallback: assume X11
        warn!("Could not detect display server; defaulting to X11");
        Ok(DisplayServer::X11)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_server_detection_succeeds() {
        // Should not panic and should return some display server
        let result = DisplayServer::detect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_server_equality() {
        assert_eq!(DisplayServer::X11, DisplayServer::X11);
        assert_eq!(DisplayServer::Wayland, DisplayServer::Wayland);
        assert_ne!(DisplayServer::X11, DisplayServer::Wayland);
    }
}
