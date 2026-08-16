/// Math utilities for the application
///
/// Vector operations, matrix transformations, and geometric calculations.
use glam::{Mat4, Vec2, Vec3};

/// Linear interpolation between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Clamp a value between min and max
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Calculate the magnitude of a 2D vector
pub fn magnitude_2d(vec: Vec2) -> f32 {
    (vec.x * vec.x + vec.y * vec.y).sqrt()
}

/// Calculate the magnitude of a 3D vector
pub fn magnitude_3d(vec: Vec3) -> f32 {
    (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt()
}

/// Normalize a 2D vector
pub fn normalize_2d(vec: Vec2) -> Vec2 {
    let mag = magnitude_2d(vec);
    if mag > 0.0 {
        Vec2::new(vec.x / mag, vec.y / mag)
    } else {
        Vec2::ZERO
    }
}

/// Normalize a 3D vector
pub fn normalize_3d(vec: Vec3) -> Vec3 {
    let mag = magnitude_3d(vec);
    if mag > 0.0 {
        Vec3::new(vec.x / mag, vec.y / mag, vec.z / mag)
    } else {
        Vec3::ZERO
    }
}

/// Dot product of two 2D vectors
pub fn dot_2d(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

/// Dot product of two 3D vectors
pub fn dot_3d(a: Vec3, b: Vec3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

/// Cross product of two 3D vectors
pub fn cross_3d(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

/// Create a 2D rotation matrix
pub fn rotation_matrix_2d(angle_radians: f32) -> Mat4 {
    let cos_a = angle_radians.cos();
    let sin_a = angle_radians.sin();
    Mat4::from_cols(
        glam::Vec4::new(cos_a, sin_a, 0.0, 0.0),
        glam::Vec4::new(-sin_a, cos_a, 0.0, 0.0),
        glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
        glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
    )
}

/// Create a translation matrix
pub fn translation_matrix(x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_translation(Vec3::new(x, y, z))
}

/// Create a scale matrix
pub fn scale_matrix(x: f32, y: f32, z: f32) -> Mat4 {
    Mat4::from_scale(Vec3::new(x, y, z))
}

/// Distance between two 2D points
pub fn distance_2d(a: Vec2, b: Vec2) -> f32 {
    magnitude_2d(b - a)
}

/// Distance between two 3D points
pub fn distance_3d(a: Vec3, b: Vec3) -> f32 {
    magnitude_3d(b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
        assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
    }

    #[test]
    fn test_magnitude_2d() {
        assert_eq!(magnitude_2d(Vec2::new(3.0, 4.0)), 5.0);
        assert_eq!(magnitude_2d(Vec2::new(0.0, 0.0)), 0.0);
    }

    #[test]
    fn test_magnitude_3d() {
        assert!((magnitude_3d(Vec3::new(1.0, 2.0, 2.0)) - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalize_2d() {
        let normalized = normalize_2d(Vec2::new(3.0, 4.0));
        assert!((normalized.x - 0.6).abs() < 0.0001);
        assert!((normalized.y - 0.8).abs() < 0.0001);
    }

    #[test]
    fn test_dot_2d() {
        assert_eq!(dot_2d(Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)), 0.0);
        assert_eq!(dot_2d(Vec2::new(1.0, 2.0), Vec2::new(3.0, 4.0)), 11.0);
    }

    #[test]
    fn test_cross_3d() {
        let a = Vec3::new(1.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 1.0, 0.0);
        let result = cross_3d(a, b);
        assert!(result.z > 0.999 && result.z < 1.001);
    }

    #[test]
    fn test_distance_2d() {
        assert_eq!(distance_2d(Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0)), 5.0);
    }

    #[test]
    fn test_distance_3d() {
        assert!((distance_3d(Vec3::ZERO, Vec3::new(1.0, 2.0, 2.0)) - 3.0).abs() < 0.0001);
    }
}
