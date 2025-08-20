@group(0) @binding(0) var original_texture: texture_2d<f32>;
@group(0) @binding(1) var position_texture: texture_2d<f32>;
@group(0) @binding(2) var spatial_index_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba32float, write>;

struct PolygonClassificationData {
   polygon_count: u32,
   total_points: u32,
   render_mode: u32,
   enable_spatial_opt: u32,
   point_data: array<vec4<f32>, 512>,
   polygon_info: array<vec4<f32>, 64>,
}

@group(0) @binding(4) var<uniform> polygon_data: PolygonClassificationData;

struct BoundsData {
    min_bounds: vec3<f32>,
    _padding1: f32,
    max_bounds: vec3<f32>,
    _padding2: f32,
}

@group(0) @binding(5) var<uniform> bounds: BoundsData;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    let texture_size = textureDimensions(original_texture);

    if coords.x >= texture_size.x || coords.y >= texture_size.y {
        return;
    }

    let original_sample = textureLoad(original_texture, coords, 0);
    let position_sample = textureLoad(position_texture, coords, 0);

    if position_sample.a == 0.0 {
        textureStore(output_texture, coords, vec4<f32>(0.0));
        return;
    }

    let world_pos = bounds.min_bounds + position_sample.xyz * (bounds.max_bounds - bounds.min_bounds);
    let original_rgb = original_sample.rgb;
    let original_class = u32(original_sample.a * 255.0);

    // Get Morton code for spatial optimisation
    let spatial_sample = textureLoad(spatial_index_texture, coords, 0);
    let morton_high = bitcast<u32>(spatial_sample.r);
    let morton_low = bitcast<u32>(spatial_sample.g);

    var final_class = original_class;
    var tests_performed = 0u;
    var tests_skipped = 0u;

    for (var i = 0u; i < polygon_data.polygon_count; i++) {
        let poly_info = polygon_data.polygon_info[i];
        let start_idx = u32(poly_info.x);
        let point_count = u32(poly_info.y);
        let new_class = u32(poly_info.z);

        if polygon_data.enable_spatial_opt == 1u {
            if is_point_near_polygon(world_pos.xz, start_idx, point_count) {
                tests_performed += 1u;
                if point_in_polygon(world_pos.xz, start_idx, point_count) {
                    final_class = new_class;
                    break;
                }
            } else {
                tests_skipped += 1u;
            }
        } else {
            tests_performed += 1u;
            if point_in_polygon(world_pos.xz, start_idx, point_count) {
                final_class = new_class;
                break;
            }
        }
    }

    let final_color = apply_render_mode(original_rgb, original_class, final_class, world_pos, coords, morton_low, tests_performed);
    textureStore(output_texture, coords, final_color);
}

fn is_point_near_polygon(point: vec2<f32>, start_idx: u32, point_count: u32) -> bool {
    if point_count == 0u { return false; }

    var min_x = polygon_data.point_data[start_idx].x;
    var max_x = min_x;
    var min_z = polygon_data.point_data[start_idx].y;
    var max_z = min_z;

    for (var i = 1u; i < point_count; i++) {
        let pt = polygon_data.point_data[start_idx + i].xy;
        min_x = min(min_x, pt.x);
        max_x = max(max_x, pt.x);
        min_z = min(min_z, pt.y);
        max_z = max(max_z, pt.y);
    }

    let margin = 1.0;
    return point.x >= (min_x - margin) && point.x <= (max_x + margin) &&
           point.y >= (min_z - margin) && point.y <= (max_z + margin);
}

fn apply_render_mode(original_rgb: vec3<f32>, original_class: u32, final_class: u32, world_pos: vec3<f32>, coords: vec2<u32>, morton_code: u32, tests_performed: u32) -> vec4<f32> {
    switch polygon_data.render_mode {
        case 0u: { // Original classification
            return vec4<f32>(classification_to_color(original_class), f32(original_class) / 255.0);
        }
        case 1u: { // Modified classification
            return vec4<f32>(classification_to_color(final_class), f32(final_class) / 255.0);
        }
        case 2u: { // RGB with classification brightness
            if length(original_rgb) > 0.1 {
                let brightness = classification_brightness(final_class);
                return vec4<f32>(original_rgb * brightness, f32(final_class) / 255.0);
            } else {
                return vec4<f32>(classification_to_color(final_class), f32(final_class) / 255.0);
            }
        }
        case 3u: { // Morton code debug
            return vec4<f32>(morton_to_debug_color(morton_code), f32(final_class) / 255.0);
        }
        case 4u: { // Spatial Debug - show which points were considered
            var was_considered = 0.0;
            for (var i = 0u; i < polygon_data.polygon_count; i++) {
                let poly_info = polygon_data.polygon_info[i];
                let start_idx = u32(poly_info.x);
                let point_count = u32(poly_info.y);

                if is_point_near_polygon(world_pos.xz, start_idx, point_count) {
                    was_considered = 1.0;
                    break;
                }
            }

            if was_considered > 0.5 {
                return vec4<f32>(1.0, 0.0, 0.0, f32(final_class) / 255.0); // Red = considered
            } else {
                return vec4<f32>(0.0, 0.0, 1.0, f32(final_class) / 255.0); // Blue = culled
            }
        }
        default: {
            return vec4<f32>(original_rgb, f32(final_class) / 255.0);
        }
    }
}

fn point_in_polygon(point: vec2<f32>, start_idx: u32, point_count: u32) -> bool {
    var inside = false;
    var j = point_count - 1u;

    for (var i = 0u; i < point_count; i++) {
        let curr_pt = polygon_data.point_data[start_idx + i].xy;
        let prev_pt = polygon_data.point_data[start_idx + j].xy;

        if ((curr_pt.y > point.y) != (prev_pt.y > point.y)) &&
           (point.x < (prev_pt.x - curr_pt.x) * (point.y - curr_pt.y) / (prev_pt.y - curr_pt.y) + curr_pt.x) {
            inside = !inside;
        }
        j = i;
    }
    return inside;
}

fn classification_brightness(classification: u32) -> f32 {
    switch classification {
        case 2u: { return 1.2; }
        case 10u: { return 1.2; }
        case 11u: { return 1.2; }
        case 12u: { return 1.2; }
        case 3u: { return 0.8; }
        case 4u: { return 0.8; }
        case 5u: { return 0.8; }
        case 6u: { return 1.1; }
        default: { return 1.0; }
    }
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

fn morton_to_debug_color(morton_code: u32) -> vec3<f32> {
    let r = f32((morton_code >> 16) & 0xFF) / 255.0;
    let g = f32((morton_code >> 8) & 0xFF) / 255.0;
    let b = f32(morton_code & 0xFF) / 255.0;
    return vec3<f32>(r, g, b);
}
