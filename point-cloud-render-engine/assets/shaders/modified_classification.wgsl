// ─────────────────────────────────────────────────────────────────────────────
// Compute Classification Shader — Phase 1 of Render Pipeline
//
// This compute pass procedurally modifies the original encoded textures based
// on polygon-based reclassification masks. It applies both spatial and logical
// filtering to determine per-point visibility and classification, writing
// results into a new output texture.
//
// Input Textures:
//   - original_texture: Encoded RGB + original classification (A)
//   - position_texture: Normalised world-space coordinates + connectivity class ID
//   - spatial_index_texture: Precomputed morton codes for spatial acceleration
//
// Uniform Data:
//   - compute_data: Packed structure with classification polygons, masks, modes
//   - bounds: World-space bounding box for position denormalisation
//
// Features:
//   - Supports two spatial filtering modes: AABB (default) and Morton (toggleable)
//   - Applies point-in-polygon masking and allows reclassification or hiding
//   - Avoids redundant operations on already-hidden points (modifier stack logic)
//   - Outputs reclassified points and/or filtered ones to `output_texture`
//   - Supports on mouse hover debug, spatial debug, morton debug, and multiple render modes
//
// Notes:
//   - Final output color encodes classification state for downstream shaders
//   - Classification ID 254 is treated as "hidden" (non-destructive hide) (note this may need to be extended upon in the future)
//   - Morton mode is optional but used for coarse-grained spatial filtering (note for long, thin polygons this is much more efficient than an AABB and in the future i would recommend a hybrid approach with some huristic)
//
// This pass sets up the modified textures used by all subsequent stages in the
// render pipeline.
// ─────────────────────────────────────────────────────────────────────────────


@group(0) @binding(0) var original_texture: texture_2d<f32>;
@group(0) @binding(1) var position_texture: texture_2d<f32>;
@group(0) @binding(2) var spatial_index_texture: texture_2d<f32>;
@group(0) @binding(3) var output_texture: texture_storage_2d<rgba32float, write>;

const MAX_IGNORE_MASK_LENGTH: u32 = 512u;
const MAXIMUM_POLYGON_POINTS: u32 = 2048u;
const MAXIMUM_POLYGONS: u32 = 512u;

struct ComputeUniformData {
    polygon_count: u32,        // 0
    total_points: u32,         // 4
    render_mode: u32,          // 8
    enable_spatial_opt: u32,   // 12  (ends at 16, aligned)

    selection_point: vec3<f32>, // 16–28
    _sel_pad: f32,             // 28–32 (vec3 padded to vec4)

    is_selecting: u32,         // 32–36
    hover_object_id: u32,      // 36–40
    _padding: vec2<u32>,       // 40–48  (fills out to next 16-byte boundary)

    point_data: array<vec4<f32>, MAXIMUM_POLYGON_POINTS>,  // starts @ 48 → aligned(16)
    polygon_info: array<vec4<f32>, MAXIMUM_POLYGONS>,
    ignore_masks: array<vec4<u32>, MAX_IGNORE_MASK_LENGTH>,
}


@group(0) @binding(4) var<uniform> compute_data: ComputeUniformData;

struct BoundsData {
    min_bounds: vec3<f32>,
    _padding1: f32,
    max_bounds: vec3<f32>,
    _padding2: f32,
}

@group(0) @binding(5) var<uniform> bounds: BoundsData;

const GRID_RESOLUTION: u32 = 1024u;
const MORTON_THRESHOLD = 500u; // Empirical threshold for Morton spatial distance.
const USE_MORTON_SPATIAL: bool = false; // Toggle: true=Morton, false=AABB

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    let texture_size = textureDimensions(original_texture);

    if coords.x >= texture_size.x || coords.y >= texture_size.y {
        return;
    }

    let original_sample = textureLoad(original_texture, coords, 0);
    let position_sample = textureLoad(position_texture, coords, 0);

    // let point_connectivity_class_id = u32(position_sample.a * 255.0);
    let point_connectivity_class_id = u32(position_sample.a * 121.0);
    let world_pos = bounds.min_bounds + position_sample.xyz * (bounds.max_bounds - bounds.min_bounds);
    let original_rgb = original_sample.rgb;
    let original_class = u32(original_sample.a * 255.0);

    // Get Morton code for spatial optimisation when using Morton mode.
    let spatial_sample = textureLoad(spatial_index_texture, coords, 0);
    let morton_high = bitcast<u32>(spatial_sample.r);
    let morton_low = bitcast<u32>(spatial_sample.g);

    var final_class = original_class;
    var found_hide_op: bool = false;

    for (var i: u32 = compute_data.polygon_count; i > 0u; i = i - 1u) {
        let real_index: u32 = i - 1u;
        let poly_info = compute_data.polygon_info[real_index];
        let start_idx = u32(poly_info.x);
        let point_count = u32(poly_info.y);
        let new_class = u32(poly_info.z);

        if compute_data.enable_spatial_opt == 1u {
            // Compile-time spatial optimization method selection.
            var should_test_polygon = false;

            if USE_MORTON_SPATIAL {
                // Morton code spatial filtering with centroid coverage.
                should_test_polygon = is_point_near_polygon_morton(
                    world_pos.xz, start_idx, point_count, bounds.min_bounds.xz, bounds.max_bounds.xz
                );
            } else {
                // AABB spatial filtering for guaranteed coverage.
                should_test_polygon = is_point_near_polygon_aabb(
                    world_pos.xz, start_idx, point_count
                );
            }

            // since this is technically a procedural modifer stack - we should not bother performing additional hide or reclasiffy ops to a point that has been previously hidden
            // otherwise intersecting polygons will undo hide ops when we reclassify, regardless of mask ids
            if should_test_polygon && !found_hide_op {
                // Here's where we check if the mask ids for the current polygon overlap AND it's inside the polygon
                // effectivly this is our masking logic per polygon in the Reclassify polygon mode
                if point_in_polygon(world_pos.xz, start_idx, point_count) {
                    // update the final class for points inside the polygon, with it's masks considered for reclassification
                    if contains_value(real_index, original_class, point_connectivity_class_id, 1u) {
                        final_class = new_class;
                    }

                    // set the final class for points inside the polygon, with masks considered to a magic number that our fragment shader will ignore and discard 'hiding' non-destructivly
                    if contains_value(real_index, original_class, point_connectivity_class_id, 0u) {
                        found_hide_op = true;
                        final_class = 254u;
                        break;
                    }

                }
            }
        } else {
            // Direct polygon testing without spatial optimization.
            if point_in_polygon(world_pos.xz, start_idx, point_count) {
                final_class = new_class;
                break;
            }
        }
    }

    let final_color = apply_render_mode(original_rgb, original_class, final_class, world_pos, coords, morton_low, morton_high, point_connectivity_class_id);
    textureStore(output_texture, coords, final_color);
}

// decode packed masks, index and mode and checks if both masks intersect (note mask id1 is the optional refinement)
fn contains_value(poly_idx: u32, mask_id0: u32, mask_id1: u32, mode: u32) -> bool {
    var index: u32 = 0u;
    loop {
        if (index >= MAX_IGNORE_MASK_LENGTH) {
            break;
        }

        let mask_vec = compute_data.ignore_masks[index];
        for (var j: u32 = 0u; j < 4u; j = j + 1u) {
            let encoded: u32 = bitcast<u32>(mask_vec[j]);

            let dec_mask_id0: u32 = encoded & 0x1FFu;
            let dec_mask_id1: u32 = (encoded >> 9u) & 0x1FFu;
            let dec_poly_idx: u32 = (encoded >> 18u) & 0x1FFu;
            let dec_mode: u32     = (encoded >> 27u) & 0x1u;

            if (dec_poly_idx == poly_idx &&
                dec_mask_id0 == mask_id0 &&
                dec_mask_id1 == mask_id1 &&
                dec_mode == mode) {
                return true;
            }
        }

        index = index + 1u;
    }

    return false;
}

/// Morton-based spatial filtering with centroid coverage for large polygons.
/// Provides fine-grained spatial discrimination but requires Morton encoding overhead.
fn is_point_near_polygon_morton(
    point: vec2<f32>,
    start_idx: u32,
    point_count: u32,
    bounds_min: vec2<f32>,
    bounds_max: vec2<f32>
) -> bool {
    if point_count == 0u { return false; }

    let query_morton = encode_morton_2d_current(point, bounds_min, bounds_max);

    // Calculate polygon centroid for interior coverage.
    // Addresses Morton spatial gaps in large polygon interiors.
    var centroid = vec2<f32>(0.0, 0.0);
    for (var i = 0u; i < point_count; i++) {
        centroid += compute_data.point_data[start_idx + i].xy;
    }
    centroid /= f32(point_count);

    // Priority check: centroid Morton distance for interior points.
    let centroid_morton = encode_morton_2d_current(centroid, bounds_min, bounds_max);
    let centroid_diff = abs(i32(query_morton) - i32(centroid_morton));
    if centroid_diff < i32(MORTON_THRESHOLD) {
        return true;
    }

    // Fallback: boundary vertex Morton distance checks.
    for (var i = 0u; i < point_count; i++) {
        let poly_point = compute_data.point_data[start_idx + i].xy;
        let poly_morton = encode_morton_2d_current(poly_point, bounds_min, bounds_max);
        let morton_diff = abs(i32(query_morton) - i32(poly_morton));
        if morton_diff < i32(MORTON_THRESHOLD) {
            return true;
        }
    }

    return false;
}

/// AABB-based spatial filtering with guaranteed coverage and no false negatives.
/// Simpler computation with O(n) vertex iteration for bounding box calculation.
fn is_point_near_polygon_aabb(
    point: vec2<f32>,
    start_idx: u32,
    point_count: u32
) -> bool {
    if point_count == 0u { return false; }

    // Calculate axis-aligned bounding box from polygon vertices.
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

    // Spatial margin to account for point cloud sampling density.
    let margin = 1.0;
    return point.x >= (min_x - margin) && point.x <= (max_x + margin) &&
           point.y >= (min_z - margin) && point.y <= (max_z + margin);
}

fn apply_render_mode(original_rgb: vec3<f32>, original_class: u32, final_class: u32, world_pos: vec3<f32>, coords: vec2<u32>, morton_low: u32, morton_high: u32, point_connectivity_class_id: u32) -> vec4<f32> {
    // we always override the colour, if the on-hover clown-pass id is provided and is the current point:
    if point_connectivity_class_id == compute_data.hover_object_id {
        return vec4<f32>(vec3(1.0, 1.0, 1.0), f32(point_connectivity_class_id));
    }

    switch compute_data.render_mode {
        case 0u: { // Original classification
            return vec4<f32>(classification_to_color(original_class), f32(point_connectivity_class_id));
        }
        case 1u: { // Modified classification
            return vec4<f32>(classification_to_color(final_class), f32(final_class));
        }
        case 2u: { // RGB
            return vec4<f32>(original_rgb, f32(point_connectivity_class_id));
        }
        case 3u: { // Morton code debug
            return vec4<f32>(morton_to_debug_color_blended(morton_low, morton_high), f32(point_connectivity_class_id));
        }
        case 4u: { // Spatial Debug - show which points were considered for processing
            var was_considered = 0.0;
            for (var i = 0u; i < compute_data.polygon_count; i++) {
                let poly_info = compute_data.polygon_info[i];
                let start_idx = u32(poly_info.x);
                let point_count = u32(poly_info.y);

                // Use same spatial filtering method as main processing loop.
                var should_test = false;
                if USE_MORTON_SPATIAL {
                    should_test = is_point_near_polygon_morton(
                        world_pos.xz, start_idx, point_count, bounds.min_bounds.xz, bounds.max_bounds.xz
                    );
                } else {
                    should_test = is_point_near_polygon_aabb(world_pos.xz, start_idx, point_count);
                }

                if should_test {
                    was_considered = 1.0;
                    break;
                }
            }

            if was_considered > 0.5 {
                return vec4<f32>(1.0, 0.0, 0.0, f32(point_connectivity_class_id)); // Red = considered
            } else {
                return vec4<f32>(0.0, 0.0, 1.0, f32(point_connectivity_class_id)); // Blue = culled
            }
        }
        case 5u: {
            return vec4<f32>(classification_to_random_color(point_connectivity_class_id), f32(point_connectivity_class_id));
        }
        default: {
            return vec4<f32>(original_rgb, f32(point_connectivity_class_id));
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

fn classification_to_random_color(classification: u32) -> vec3<f32> {
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

fn classification_to_color(classification: u32) -> vec3<f32> {
    switch classification {
        case 0u: { return vec3<f32>(0.85, 0.85, 0.85); }     // never classified
        case 1u: { return vec3<f32>(0.73, 0.73, 0.73); }     // unclassified
        case 2u: { return vec3<f32>(1.0, 0.6, 0.0); }        // sidewalk
        case 3u: { return vec3<f32>(0.28, 0.70, 0.28); }     // low vegetation
        case 4u: { return vec3<f32>(0.0, 0.8, 0.0); }        // medium vegetation
        case 5u: { return vec3<f32>(0.0, 0.6, 0.0); }        // high vegetation
        case 6u: { return vec3<f32>(0.92, 1.0, 0.0); }       // buildings
        case 8u: { return vec3<f32>(0.2, 0.0, 1.0); }        // street furniture
        case 10u: { return vec3<f32>(1.0, 1.0, 1.0); }       // street markings
        case 11u: { return vec3<f32>(0.18, 0.18, 0.18); }    // street surface
        case 13u: { return vec3<f32>(1.0, 0.95, 0.0); }      // non-permanent
        case 15u: { return vec3<f32>(1.0, 0.0, 0.0); }       // cars
        case 20u: { return vec3<f32>(0.7, 0.5, 0.8); }       // highlight
        default: { return vec3<f32>(0.5, 0.5, 0.5); }        // fallback for unknown classifications
    }
}

fn morton_to_debug_color_blended(morton_high: u32, morton_low: u32) -> vec3<f32> {
    let combined1 = morton_high ^ (morton_low >> 16);
    let combined2 = morton_low ^ (morton_high >> 16);

    let r = f32((combined1 >> 16) & 0xFF) / 255.0;
    let g = f32((combined1 >> 8) & 0xFF) / 255.0;
    let b = f32(combined2 & 0xFF) / 255.0;

    return vec3<f32>(r, g, b);
}

// Helper function for Morton encoding
fn morton_part_1by1(n: u32) -> u32 {
    var result = n & 0x0000FFFFu;
    result = (result ^ (result << 8u)) & 0x00FF00FFu;
    result = (result ^ (result << 4u)) & 0x0F0F0F0Fu;
    result = (result ^ (result << 2u)) & 0x33333333u;
    result = (result ^ (result << 1u)) & 0x55555555u;
    return result;
}

fn morton_encode_2d_optimized(x: u32, z: u32) -> u32 {
    return morton_part_1by1(x) | (morton_part_1by1(z) << 1u);
}

fn encode_morton_2d_current(point: vec2<f32>, bounds_min: vec2<f32>, bounds_max: vec2<f32>) -> u32 {
    // Normalise coordinates to match bounds normalisation logic.
    let norm_x = (point.x - bounds_min.x) / (bounds_max.x - bounds_min.x);
    let norm_z = (point.y - bounds_min.y) / (bounds_max.y - bounds_min.y);

    // Calculate grid coordinates for Morton encoding.
    let grid_x = u32(clamp(norm_x * f32(GRID_RESOLUTION - 1u), 0.0, f32(GRID_RESOLUTION - 1u)));
    let grid_z = u32(clamp(norm_z * f32(GRID_RESOLUTION - 1u), 0.0, f32(GRID_RESOLUTION - 1u)));

    return morton_encode_2d_optimized(grid_x, grid_z);
}
