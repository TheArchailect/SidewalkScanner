@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;

struct EDLSettings {
    radius: f32,
    strength: f32,
    ambient_boost: f32,
    contrast: f32,
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

    let sample_offsets = array<vec2<i32>, 8>(
        vec2<i32>(0, -i32(settings.radius)),      // Up
        vec2<i32>(i32(settings.radius * 0.707), -i32(settings.radius * 0.707)),  // Up-Right
        vec2<i32>(i32(settings.radius), 0),       // Right
        vec2<i32>(i32(settings.radius * 0.707), i32(settings.radius * 0.707)),   // Down-Right
        vec2<i32>(0, i32(settings.radius)),       // Down
        vec2<i32>(-i32(settings.radius * 0.707), i32(settings.radius * 0.707)),  // Down-Left
        vec2<i32>(-i32(settings.radius), 0),      // Left
        vec2<i32>(-i32(settings.radius * 0.707), -i32(settings.radius * 0.707)) // Up-Left
    );


    for (var i = 0u; i < 8u; i++) {
        let neighbor_coords = pixel_coords + sample_offsets[i];

        // Check bounds
        if neighbor_coords.x >= 0 && neighbor_coords.x < i32(screen_size.x) &&
           neighbor_coords.y >= 0 && neighbor_coords.y < i32(screen_size.y) {

            let neighbor_sample = textureLoad(screen_texture, neighbor_coords, 0);
            let neighbor_depth = neighbor_sample.a;

            if neighbor_depth > 0.0 {
                // let log_current = log2(max(current_depth, 0.001));
                // let log_neighbor = log2(max(neighbor_depth, 0.001));
                // let depth_diff = max(0.0, log_current - log_neighbor);
                let depth_diff = max(0.0, current_depth - neighbor_depth);

                total_obscurance += depth_diff;
                valid_samples += 1u;
            }
        }
    }

    // Calculate shade factor
    var shade = 1.0;
    if valid_samples > 0u {
        if (total_obscurance < 0.5) {
            let avg_obscurance = total_obscurance / f32(valid_samples);
            shade = exp(-avg_obscurance * settings.strength);
            shade = clamp(shade, 0.01, 1.0);
        }
    }

    return vec4<f32>(current_sample.rgb * shade, 1.0);
}
