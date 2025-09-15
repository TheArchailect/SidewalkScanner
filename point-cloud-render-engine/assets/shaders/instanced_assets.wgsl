#import bevy_pbr::view_transformations::position_world_to_clip

@group(2) @binding(0) var asset_position_texture: texture_2d<f32>;
@group(2) @binding(1) var asset_position_sampler: sampler;
@group(2) @binding(2) var asset_color_texture: texture_2d<f32>;
@group(2) @binding(3) var asset_color_sampler: sampler;

struct CameraUniforms {
    position: vec3<f32>,
    _padding: f32,
}

@group(2) @binding(4) var<uniform> camera: CameraUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(3) i_position: vec3<f32>,
    @location(4) i_rotation: vec4<f32>,
    @location(5) i_uv_bounds: vec4<f32>,
    @location(6) i_point_count_pad: vec4<f32>,
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) quad_pos: vec2<f32>,
}

fn quat_rotate_vec3(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    let qvec = q.xyz;
    let uv = cross(qvec, v);
    let uuv = cross(qvec, uv);
    return v + ((uv * q.w) + uuv) * 2.0;
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let point_index = vertex.vertex_index / 6u;
    let quad_vertex = vertex.vertex_index % 6u;

    var quad_pos: vec2<f32>;
    switch quad_vertex {
        case 0u: { quad_pos = vec2<f32>(-0.5, -0.5); }
        case 1u: { quad_pos = vec2<f32>( 0.5, -0.5); }
        case 2u: { quad_pos = vec2<f32>( 0.5,  0.5); }
        case 3u: { quad_pos = vec2<f32>(-0.5, -0.5); }
        case 4u: { quad_pos = vec2<f32>( 0.5,  0.5); }
        default: { quad_pos = vec2<f32>(-0.5,  0.5); }
    }
    out.quad_pos = quad_pos;

    let points_per_side = u32(sqrt(vertex.i_point_count_pad.x));
    if points_per_side == 0u {
        out.clip_position = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return out;
    }

    let atlas_x = point_index % points_per_side;
    let atlas_y = point_index / points_per_side;

    let local_uv = vec2<f32>(
        (f32(atlas_x) + 0.5) / f32(points_per_side),
        (f32(atlas_y) + 0.5) / f32(points_per_side)
    );

    let atlas_uv = mix(vertex.i_uv_bounds.xy, vertex.i_uv_bounds.zw, local_uv);

    let pos_sample = textureSampleLevel(asset_position_texture, asset_position_sampler, atlas_uv, 0.0);

    if pos_sample.a < 0.1 {
        out.clip_position = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return out;
    }

    // Get local position from texture (normalized -1 to 1)
    let local_pos = pos_sample.rgb * 2.0 - 1.0;

    // Apply instance rotation to local position
    let rotated_pos = quat_rotate_vec3(vertex.i_rotation, local_pos);

    // Transform to world space
    let world_pos = vertex.i_position + rotated_pos;

    // Billboard calculation
    let camera_pos = camera.position;
    let to_camera = normalize(world_pos - camera_pos);
    let right = normalize(cross(to_camera, vec3<f32>(0.0, 1.0, 0.0)));
    let up = cross(right, to_camera);

    let distance = length(world_pos - camera_pos);
    let point_size = mix(0.02, 0.15, clamp((distance - 2.0) / 48.0, 0.0, 1.0));

    let billboard_offset = right * quad_pos.x * point_size + up * quad_pos.y * point_size;
    let final_world_pos = world_pos + billboard_offset;

    out.clip_position = position_world_to_clip(final_world_pos);

    let color_sample = textureSampleLevel(asset_color_texture, asset_color_sampler, atlas_uv, 0.0);
    out.color = color_sample.rgb;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = length(in.quad_pos); // 0 at center, ~0.707 at corners
    let radius = 0.5;

    if (r > radius) {
        discard;  // hard circle cutout
    }

    return vec4<f32>(in.color, 0.0);
}
