use glam::Vec2;

pub fn cursor_to_ndc(cursor_px: Vec2, viewport_px: Vec2) -> Vec2 {
    Vec2::new(
        cursor_px.x / (viewport_px.x * 0.5) - 1.0,
        1.0 - cursor_px.y / (viewport_px.y * 0.5),
    )
}

pub fn pixel_delta_to_ndc(delta_px: Vec2, viewport_px: Vec2) -> Vec2 {
    Vec2::new(
        delta_px.x / (viewport_px.x * 0.5),
        -delta_px.y / (viewport_px.y * 0.5),
    )
}
