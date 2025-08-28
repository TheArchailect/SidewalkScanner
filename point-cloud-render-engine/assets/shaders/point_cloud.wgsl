#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct PointCloudMaterial {
    params: array<vec4<f32>, 2>,
}

@group(2) @binding(0) var position_texture: texture_2d<f32>;
@group(2) @binding(1) var position_sampler: sampler;
@group(2) @binding(2) var final_texture: texture_2d<f32>;
@group(2) @binding(3) var final_sampler: sampler;
@group(2) @binding(4) var<uniform> material: PointCloudMaterial;

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
    let texture_size = material.params[0].w;
    let x_coord = point_index % u32(texture_size);
    let y_coord = point_index / u32(texture_size);

    let uv = vec2<f32>(
        (f32(x_coord) + 0.5) / texture_size,
        (f32(y_coord) + 0.5) / texture_size
    );

    // Sample position
    let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

    if pos_sample.a == 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    // Denormalize to world space
    let norm_pos = pos_sample.rgb;
    let min_pos = material.params[0].xyz;
    let max_pos = material.params[1].xyz;
    let world_pos = min_pos + norm_pos * (max_pos - min_pos);

    // Transform to clip space
    let clip_pos = mesh_position_local_to_clip(mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ), vec4<f32>(world_pos, 1.0));

    if clip_pos.w <= 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    out.clip_position = clip_pos;

    // Sample final color from compute shader output
    let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
    out.color = textureLoad(final_texture, tex_coord, 0);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = in.color.a;
    let normalized_depth = clamp((depth - 1.0) / (50.0 - 1.0), 0.0, 1.0);
    return vec4<f32>(in.color.xyz, normalized_depth);
}
