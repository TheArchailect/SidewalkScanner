#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}
struct PointCloudMaterial {
    params: array<vec4<f32>, 3>,  // params[0] = min_bounds+size, params[1] = max_bounds, params[2] = camera_pos+padding
}

@group(2) @binding(0) var position_texture: texture_2d<f32>;
@group(2) @binding(1) var position_sampler: sampler;
@group(2) @binding(2) var final_texture: texture_2d<f32>;
@group(2) @binding(3) var final_sampler: sampler;

@group(2) @binding(4) var depth_texture: texture_2d<f32>;
@group(2) @binding(5) var depth_sampler: sampler;

@group(2) @binding(6) var<uniform> material: PointCloudMaterial;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) depth: f32,
    @location(2) connectivity_class: u32,
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Calculate point index and quad vertex from vertex_index
    let point_index = vertex.vertex_index / 6u;
    let quad_vertex = vertex.vertex_index % 6u;

    // Generate quad vertices for this point
    var quad_pos: vec2<f32>;
    switch quad_vertex {
        case 0u: { quad_pos = vec2<f32>(-0.5, -0.5); } // Bottom left
        case 1u: { quad_pos = vec2<f32>( 0.5, -0.5); } // Bottom right
        case 2u: { quad_pos = vec2<f32>( 0.5,  0.5); } // Top right
        case 3u: { quad_pos = vec2<f32>(-0.5, -0.5); } // Bottom left (triangle 2)
        case 4u: { quad_pos = vec2<f32>( 0.5,  0.5); } // Top right (triangle 2)
        default: { quad_pos = vec2<f32>(-0.5,  0.5); } // Top left (triangle 2)
    }

    let texture_size = material.params[0].w;
    let x_coord = point_index % u32(texture_size);
    let y_coord = point_index / u32(texture_size);

    let uv = vec2<f32>(
        (f32(x_coord) + 0.5) / texture_size,
        (f32(y_coord) + 0.5) / texture_size
    );

    // Sample position
    let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

    // Denormalize to world space
    let norm_pos = pos_sample.rgb;
    let min_pos = material.params[0].xyz;
    let max_pos = material.params[1].xyz;
    let world_pos = min_pos + norm_pos * (max_pos - min_pos);

    // Billboard towards camera
    let camera = material.params[2].xyz;
    let to_camera = normalize(world_pos - camera);
    let right = normalize(cross(to_camera, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, to_camera);

    // Apply interpolated billboarded quad offset
    let point_size_min = 0.085;
    let point_size_max = 0.4;

    let dist_min = 10.0;
    let dist_max = 100.0;

    let dist = distance(world_pos, camera);

    // normalised 0..1 factor based on distance
    let t = clamp((dist - dist_min) / (dist_max - dist_min), 0.0, 1.0);

    // linear interpolation between min and max point sizes
    let point_size = mix(point_size_min, point_size_max, t);

    let offset = right * quad_pos.x * point_size + up * quad_pos.y * point_size;
    let final_world_pos = world_pos + offset;

    // Transform to clip space
    let clip_pos = mesh_position_local_to_clip(mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    ), vec4<f32>(final_world_pos, 1.0));

    out.clip_position = clip_pos;

    // Sample final color
    let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
    let current_rgba = textureLoad(final_texture, tex_coord, 0);
    out.color = current_rgba.rgb;
    out.connectivity_class = u32(current_rgba.a);
    out.depth = textureLoad(depth_texture, tex_coord, 0).r;
    return out;
}

fn classification_to_color(classification: u32) -> vec3<f32> {
    let c = classification & 255u;
    let hash1 = (c * 73u) % 255u;
    let hash2 = (c * 151u + 17u) % 255u;
    let hash3 = (c * 211u + 37u) % 255u;

    return vec3<f32>(
        f32(hash1) / 255.0,
        f32(hash2) / 255.0,
        f32(hash3) / 255.0
    );
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = in.depth;
    let near = 0.1;
    let far = 500.0;
    let norm_depth = (depth - near) / (far - near);
    let depth_rb = mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.1), norm_depth);

    // return vec4<f32>(depth_rb, depth);
    return vec4<f32>(in.color, depth);
    // return vec4<f32>(classification_to_color(in.connectivity_class), depth);
}
