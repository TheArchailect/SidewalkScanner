@group(0) @binding(0) var position_texture: texture_2d<f32>;
@group(0) @binding(1) var color_input: texture_2d<f32>;
// @group(0) @binding(2) var color_output: texture_storage_2d<rgba32float, write>;
@group(0) @binding(2) var color_output: texture_storage_2d<r32float, write>;

struct EDLUniforms {
    view_matrix: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding1: f32,
    bounds_min: vec3<f32>,
    _padding2: f32,
    bounds_max: vec3<f32>,
    _padding3: f32,
}

fn normalizeRange(x: f32, n: f32, m: f32) -> f32 {
    return clamp((x - n) / (m - n), 0.0, 1.0);
}

fn normalizeDepthLog(d: f32, near: f32, far: f32) -> f32 {
    let nd = log(d / near) / log(far / near);
    return clamp(nd, 0.0, 1.0);
}


@group(0) @binding(3) var<uniform> edl_params: EDLUniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    let texture_size = textureDimensions(color_input);

    if coords.x >= texture_size.x || coords.y >= texture_size.y {
        return;
    }

    var depth_camera = vec3<f32>(edl_params.camera_pos.x, edl_params.camera_pos.y, edl_params.camera_pos.z);
    let position_sample = textureLoad(position_texture, coords, 0);
    let color_sample = textureLoad(color_input, coords, 0);
    var final_depth = 0.0;

    // If we have a valid point at this pixel
    if position_sample.a > 0.0 {
        let world_pos = edl_params.bounds_min + position_sample.xyz * (edl_params.bounds_max - edl_params.bounds_min);
        final_depth = length(world_pos - depth_camera);
    } else {
        // Interpolate depth from nearby valid points
        var total_depth = 0.0;
        var count = 0u;

        for (var dx = -3; dx <= 3; dx++) {
            for (var dy = -3; dy <= 3; dy++) {
                if dx == 0 && dy == 0 { continue; }

                let nx = i32(coords.x) + dx;
                let ny = i32(coords.y) + dy;

                if nx >= 0 && nx < i32(texture_size.x) && ny >= 0 && ny < i32(texture_size.y) {
                    let neighbor_pos = textureLoad(position_texture, vec2<u32>(u32(nx), u32(ny)), 0);
                    let neighbor_world = edl_params.bounds_min + neighbor_pos.xyz * (edl_params.bounds_max - edl_params.bounds_min);
                    let neighbor_depth = length(neighbor_world - depth_camera);
                    total_depth += neighbor_depth;
                    count += 1u;
                }
            }
        }

        if count > 0u {
            final_depth = total_depth / f32(count);
        }
    }

    // depth buffer hack, this needs to be re-writen with some more complex logic
    let norm_depth = normalizeRange(final_depth, 0.0 , edl_params.camera_pos.y * 2.0);
    // let norm_depth = normalizeDepthLog(final_depth, 0.1, edl_params.camera_pos.y * 2.0);
    let noise = fract(sin(dot(vec2<f32>(coords), vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let dithered_depth = norm_depth + (noise - 0.5) * 0.001;

    // let near = 0.1;
    // let far = edl_params.camera_pos.y;
    // let norm_depth = (final_depth - near) / (far - near);

    // Store color in RGB, depth in alpha for fragment shader
    // textureStore(color_output, coords, vec4<f32>(color_sample.rgb, dithered_depth));
    textureStore(color_output, coords, vec4<f32>(dithered_depth, 0.0, 0.0, 0.0));
}
