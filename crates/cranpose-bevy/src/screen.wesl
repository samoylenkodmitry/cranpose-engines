// Cranpose stores a screen's colors sRGB-encoded; the frame takes linear
// light. A screen gives off its own light, so nothing lights it.
@group(constants::MATERIAL_BIND_GROUP) @binding(0) var screen: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var screen_sampler: sampler;

struct Fragment {
    @location(2) uv: vec2<f32>,
}

fn linear_from_srgb(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + 0.055) / 1.055, vec3(2.4));
    return select(high, low, color <= vec3(0.04045));
}

@fragment
fn fragment(input: Fragment) -> @location(0) vec4<f32> {
    let stored = textureSample(screen, screen_sampler, input.uv);
    return vec4(linear_from_srgb(stored.rgb), 1.0);
}
