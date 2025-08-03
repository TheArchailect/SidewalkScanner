#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct PointCloudMaterial {
    params: array<vec4<f32>, 2>,
}

@group(1) @binding(0) var position_texture: texture_2d<f32>;
@group(1) @binding(1) var position_sampler: sampler;
@group(1) @binding(2) var metadata_texture: texture_2d<f32>;
@group(1) @binding(3) var metadata_sampler: sampler;
@group(1) @binding(4) var<uniform> material: PointCloudMaterial;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let point_index = u32(vertex.position.x);

    // Convert point index to texture coordinates
    let texture_size = material.params[0].w;
    let x_coord = point_index % u32(texture_size);
    let y_coord = point_index / u32(texture_size);

    // Sample position from texture using UV
    let uv = vec2<f32>(
        (f32(x_coord) + 0.5) / texture_size,
        (f32(y_coord) + 0.5) / texture_size
    );
    let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

    // Skip invalid points (alpha == 0)
    if pos_sample.a == 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0); // Move far behind camera
        out.color = vec4<f32>(0.0);
        return out;
    }

    // Denormalize coordinates
    let norm_pos = pos_sample.rgb;
    let min_pos = material.params[0].xyz;
    let max_pos = material.params[1].xyz;
    let range = max_pos - min_pos;
    let world_pos = min_pos + norm_pos * range;

    // Clamp positions to prevent extreme values
    let clamped_pos = clamp(world_pos, min_pos - range * 0.1, max_pos + range * 0.1);

    // Transform to clip space
    let clip_pos = mesh_position_local_to_clip(mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ), vec4<f32>(clamped_pos, 1.0));


    // Ensure w component is valid
    if clip_pos.w <= 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    out.clip_position = clip_pos;

    // Sample metadata using exact pixel coordinates
    let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
    let meta_sample = textureLoad(metadata_texture, tex_coord, 0);
    let classification = u32(meta_sample.r * 255.0);
    let intensity = meta_sample.g;

    // Convert classification to color
    out.color = classification_to_color(classification, intensity);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

fn classification_to_color(classification: u32, intensity: f32) -> vec4<f32> {
    let c = classification & 255u;
    let hash1 = (c * 73u) % 255u;
    let hash2 = (c * 151u + 17u) % 255u;
    let hash3 = (c * 211u + 37u) % 255u;

    let r = f32(hash1) / 255.0;
    let g = f32(hash2) / 255.0;
    let b = f32(hash3) / 255.0;

    let brightness = 0.7 + intensity * 0.3;
    return vec4<f32>(r * brightness, g * brightness, b * brightness, 1.0);
}
