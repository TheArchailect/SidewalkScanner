@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct EDLSettings {
    radius: f32,
    strength: f32,
}

@group(0) @binding(2) var<uniform> settings: EDLSettings;

// Input structure for the fragment shader
struct FragmentInput {
    @builtin(position) position: vec4<f32>,
}

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
    let screen_size = textureDimensions(screen_texture);
    let uv = input.position.xy / vec2<f32>(screen_size);
    let texel_size = 1.0 / vec2<f32>(screen_size);

    // Use textureLoad instead of textureSample for nearest neighbor access
    let pixel_coords = vec2<i32>(input.position.xy);
    let current_sample = textureLoad(screen_texture, pixel_coords, 0);
    let current_depth = current_sample.a;

    // Skip background pixels
    if current_depth <= 0.0 {
        return current_sample;
    }

    // EDL calculation - sample screen-space neighbors using textureLoad
    var total_obscurance = 0.0;
    var valid_samples = 0u;

    // Sample 4 orthogonal neighbors
    let sample_offsets = array<vec2<i32>, 4>(
        vec2<i32>(0, -i32(settings.radius)),  // Up
        vec2<i32>(-i32(settings.radius), 0),  // Left
        vec2<i32>(i32(settings.radius), 0),   // Right
        vec2<i32>(0, i32(settings.radius))    // Down
    );

    for (var i = 0u; i < 4u; i++) {
        let neighbor_coords = pixel_coords + sample_offsets[i];

        // Check bounds
        if neighbor_coords.x >= 0 && neighbor_coords.x < i32(screen_size.x) &&
           neighbor_coords.y >= 0 && neighbor_coords.y < i32(screen_size.y) {

            let neighbor_sample = textureLoad(screen_texture, neighbor_coords, 0);
            let neighbor_depth = neighbor_sample.a;

            if neighbor_depth > 0.0 {
                // Calculate depth difference in log space
                let log_current = log2(max(current_depth, 0.001));
                let log_neighbor = log2(max(neighbor_depth, 0.001));
                let depth_diff = max(0.0, log_current - log_neighbor);

                total_obscurance += depth_diff;
                valid_samples += 1u;
            }
        }
    }

    // Calculate shade factor
    var shade = 1.0;
    if valid_samples > 0u {
        let avg_obscurance = total_obscurance / f32(valid_samples);
        shade = exp(-avg_obscurance * 300.0 * settings.strength);
        shade = clamp(shade, 0.1, 1.0);
    }

    return vec4<f32>(current_sample.rgb * shade, current_sample.a);
}
