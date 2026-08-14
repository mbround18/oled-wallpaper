// Celestial body rendering shader
// Renders circles for sun and planets with colors

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) radius: f32,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) distance_from_center: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Convert world position to clip space
    // For now, just pass through the position
    out.position = vec4<f32>(in.position.xy / 500.0, in.position.z, 1.0);
    out.color = in.color;
    out.distance_from_center = 0.0;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return the color of the celestial body
    // Apply alpha for fade effects
    let alpha = in.color.w;
    return vec4<f32>(in.color.xyz, alpha);
}
