@group(0) @binding(0) var original_texture: texture_2d<f32>;
@group(0) @binding(1) var position_texture: texture_2d<f32>;
@group(0) @binding(2) var spatial_index_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba32float, write>;

struct ComputeUniformData {
   polygon_count: u32,
   total_points: u32,
   render_mode: u32,
   enable_spatial_opt: u32,
   selection_point: vec2<f32>,
   is_selecting: u32,
   _padding: u32,
   point_data: array<vec4<f32>, 512>,
   polygon_info: array<vec4<f32>, 64>,
}

@group(0) @binding(4) var<uniform> compute_data: ComputeUniformData;

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

    for (var i = 0u; i < compute_data.polygon_count; i++) {
        let poly_info = compute_data.polygon_info[i];
        let start_idx = u32(poly_info.x);
        let point_count = u32(poly_info.y);
        let new_class = u32(poly_info.z);

        if compute_data.enable_spatial_opt == 1u {
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

    var min_x = compute_data.point_data[start_idx].x;
    var max_x = min_x;
    var min_z = compute_data.point_data[start_idx].y;
    var max_z = min_z;

    for (var i = 1u; i < point_count; i++) {
        let pt = compute_data.point_data[start_idx + i].xy;
        min_x = min(min_x, pt.x);
        max_x = max(max_x, pt.x);
        min_z = min(min_z, pt.y);
        max_z = max(max_z, pt.y);
    }

    let margin = 1.0;
    return point.x >= (min_x - margin) && point.x <= (max_x + margin) &&
           point.y >= (min_z - margin) && point.y <= (max_z + margin);
}

fn find_nearest_class_at_point(selection_point: vec2<f32>) -> u32 {
    let texture_size = textureDimensions(original_texture);
    let search_radius = 5.0;

    var closest_distance = 999999.0;
    var closest_class = 2u;

    // Convert to texture coordinates
    let norm_x = (selection_point.x - bounds.min_bounds.x) / (bounds.max_bounds.x - bounds.min_bounds.x);
    let norm_z = (selection_point.y - bounds.min_bounds.z) / (bounds.max_bounds.z - bounds.min_bounds.z);
    let center_x = u32(clamp(norm_x, 0.0, 1.0) * f32(texture_size.x - 1u));
    let center_y = u32(clamp(norm_z, 0.0, 1.0) * f32(texture_size.y - 1u));

    // Search in 10x10 pixel area
    for (var dy = -5i; dy <= 5i; dy++) {
        for (var dx = -5i; dx <= 5i; dx++) {
            let sample_x = i32(center_x) + dx;
            let sample_y = i32(center_y) + dy;

            if sample_x >= 0 && sample_x < i32(texture_size.x) &&
               sample_y >= 0 && sample_y < i32(texture_size.y) {

                let sample_coords = vec2<u32>(u32(sample_x), u32(sample_y));
                let position_sample = textureLoad(position_texture, sample_coords, 0);

                if position_sample.a > 0.0 {
                    let world_pos = bounds.min_bounds + position_sample.xyz * (bounds.max_bounds - bounds.min_bounds);
                    let distance = length(world_pos.xz - selection_point);

                    if distance < closest_distance {
                        let class_sample = textureLoad(original_texture, sample_coords, 0);
                        let point_class = u32(class_sample.a * 255.0);

                        if point_class > 0u {
                            closest_distance = distance;
                            closest_class = point_class;
                        }
                    }
                }
            }
        }
    }

    return closest_class;
}

fn apply_render_mode(original_rgb: vec3<f32>, original_class: u32, final_class: u32, world_pos: vec3<f32>, coords: vec2<u32>, morton_code: u32, tests_performed: u32) -> vec4<f32> {
    switch compute_data.render_mode {
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
            for (var i = 0u; i < compute_data.polygon_count; i++) {
                let poly_info = compute_data.polygon_info[i];
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
        case 5u: { // Class selection
            if compute_data.is_selecting == 1u {
                let selected_class = find_nearest_class_at_point(compute_data.selection_point);

                if original_class == selected_class {
                    return vec4<f32>(0.0, 1.0, 0.0, f32(original_class) / 255.0); // Green
                }
                return vec4<f32>(original_rgb * 0.3, f32(original_class) / 255.0); // Dimmed others
            }
            return vec4<f32>(original_rgb, f32(original_class) / 255.0);
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
        let curr_pt = compute_data.point_data[start_idx + i].xy;
        let prev_pt = compute_data.point_data[start_idx + j].xy;

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
