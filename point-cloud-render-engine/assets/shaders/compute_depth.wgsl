@group(0) @binding(0) var position_texture: texture_2d<f32>;
@group(0) @binding(1) var color_input: texture_2d<f32>;
@group(0) @binding(2) var color_output: texture_storage_2d<rgba32float, write>;

struct EDLUniforms {
    view_matrix: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding1: f32,
    bounds_min: vec3<f32>,
    _padding2: f32,
    bounds_max: vec3<f32>,
    _padding3: f32,
}

@group(0) @binding(3) var<uniform> edl_params: EDLUniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = global_id.xy;
    let texture_size = textureDimensions(color_input);

    if coords.x >= texture_size.x || coords.y >= texture_size.y {
        return;
    }

    var depth_camera = vec3<f32>(edl_params.camera_pos.x, 10.0, edl_params.camera_pos.z);
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
                    if neighbor_pos.a > 0.0 {
                        let neighbor_world = edl_params.bounds_min + neighbor_pos.xyz * (edl_params.bounds_max - edl_params.bounds_min);
                        let neighbor_depth = length(neighbor_world - depth_camera);
                        total_depth += neighbor_depth;
                        count += 1u;
                    }
                }
            }
        }

        if count > 0u {
            final_depth = total_depth / f32(count);
        }
    }

    // Store color in RGB, depth in alpha for fragment shader
    textureStore(color_output, coords, vec4<f32>(color_sample.rgb, final_depth));
}

// // // === RADIAL DEPTH CONTROL CONSTANTS ===
// // // Adjust these values to control the radial depth behavior

// // // Fixed Y height for the depth camera (overrides actual camera Y)
// // const DEPTH_CAMERA_Y: f32 = 10.0;

// // // Auto-calculate depth range from scene bounds (true) or use manual range (false)
// // const AUTO_DEPTH_RANGE: bool = true;

// // // Manual depth range (only used when AUTO_DEPTH_RANGE = false)
// // const MANUAL_MIN_DEPTH: f32 = 0.0;
// // const MANUAL_MAX_DEPTH: f32 = 100.0;

// // // Depth remapping curve type
// // // 0: Linear, 1: Exponential (emphasize close), 2: Logarithmic (emphasize far), 3: Sigmoid
// // const DEPTH_CURVE: i32 = 0;

// // // Curve strength (only used for non-linear curves)
// // const CURVE_STRENGTH: f32 = 2.0;

// // // ============================================

// // @group(0) @binding(0) var position_texture: texture_2d<f32>;
// // @group(0) @binding(1) var color_input: texture_2d<f32>;
// // @group(0) @binding(2) var color_output: texture_storage_2d<rgba32float, write>;

// // struct EDLUniforms {
// //     view_matrix: mat4x4<f32>,
// //     camera_pos: vec3<f32>,
// //     _padding1: f32,
// //     bounds_min: vec3<f32>,
// //     _padding2: f32,
// //     bounds_max: vec3<f32>,
// //     _padding3: f32,
// // }

// // @group(0) @binding(3) var<uniform> edl_params: EDLUniforms;

// // // Calculate the radial depth range from scene bounds
// // fn calculate_radial_depth_bounds(depth_camera: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> vec2<f32> {
// //     // Test all 8 corners of the bounding box to find true min/max radial distances
// //     let corners = array<vec3<f32>, 8>(
// //         vec3<f32>(bounds_min.x, bounds_min.y, bounds_min.z),
// //         vec3<f32>(bounds_max.x, bounds_min.y, bounds_min.z),
// //         vec3<f32>(bounds_min.x, bounds_max.y, bounds_min.z),
// //         vec3<f32>(bounds_max.x, bounds_max.y, bounds_min.z),
// //         vec3<f32>(bounds_min.x, bounds_min.y, bounds_max.z),
// //         vec3<f32>(bounds_max.x, bounds_min.y, bounds_max.z),
// //         vec3<f32>(bounds_min.x, bounds_max.y, bounds_max.z),
// //         vec3<f32>(bounds_max.x, bounds_max.y, bounds_max.z)
// //     );

// //     var min_dist = length(corners[0] - depth_camera);
// //     var max_dist = min_dist;

// //     for (var i = 1; i < 8; i++) {
// //         let dist = length(corners[i] - depth_camera);
// //         min_dist = min(min_dist, dist);
// //         max_dist = max(max_dist, dist);
// //     }

// //     return vec2<f32>(min_dist, max_dist);
// // }

// // // Apply depth curve remapping
// // fn apply_depth_curve(normalized_depth: f32) -> f32 {
// //     switch DEPTH_CURVE {
// //         case 1: {
// //             // Exponential - emphasizes close objects
// //             return 1.0 - exp(-normalized_depth * CURVE_STRENGTH);
// //         }
// //         case 2: {
// //             // Logarithmic - emphasizes far objects
// //             return log(1.0 + normalized_depth * (exp(CURVE_STRENGTH) - 1.0)) / CURVE_STRENGTH;
// //         }
// //         case 3: {
// //             // Sigmoid - smooth S-curve
// //             let shifted = (normalized_depth - 0.5) * CURVE_STRENGTH;
// //             return 0.5 + tanh(shifted) * 0.5;
// //         }
// //         default: {
// //             // Linear (no curve)
// //             return normalized_depth;
// //         }
// //     }
// // }

// // @compute @workgroup_size(8, 8, 1)
// // fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
// //     let coords = global_id.xy;
// //     let texture_size = textureDimensions(color_input);

// //     if coords.x >= texture_size.x || coords.y >= texture_size.y {
// //         return;
// //     }

// //     // Create depth camera with fixed Y height
// //     var depth_camera = vec3<f32>(edl_params.camera_pos.x, DEPTH_CAMERA_Y, edl_params.camera_pos.z);

// //     let position_sample = textureLoad(position_texture, coords, 0);
// //     let color_sample = textureLoad(color_input, coords, 0);

// //     // Calculate depth range for normalization
// //     var depth_range: vec2<f32>;
// //     if AUTO_DEPTH_RANGE {
// //         depth_range = calculate_radial_depth_bounds(depth_camera, edl_params.bounds_min, edl_params.bounds_max);
// //     } else {
// //         depth_range = vec2<f32>(MANUAL_MIN_DEPTH, MANUAL_MAX_DEPTH);
// //     }

// //     var final_normalized_depth = 0.0;

// //     // If we have a valid point at this pixel
// //     if position_sample.a > 0.0 {
// //         let world_pos = edl_params.bounds_min + position_sample.xyz * (edl_params.bounds_max - edl_params.bounds_min);
// //         let raw_depth = length(world_pos - depth_camera);

// //         // Normalize to [0,1] range
// //         let normalized = clamp((raw_depth - depth_range.x) / (depth_range.y - depth_range.x), 0.0, 1.0);

// //         // Apply curve remapping
// //         final_normalized_depth = apply_depth_curve(normalized);
// //     } else {
// //         // Interpolate depth from nearby valid points
// //         var total_normalized_depth = 0.0;
// //         var count = 0u;

// //         for (var dx = -3; dx <= 3; dx++) {
// //             for (var dy = -3; dy <= 3; dy++) {
// //                 if dx == 0 && dy == 0 { continue; }

// //                 let nx = i32(coords.x) + dx;
// //                 let ny = i32(coords.y) + dy;

// //                 if nx >= 0 && nx < i32(texture_size.x) && ny >= 0 && ny < i32(texture_size.y) {
// //                     let neighbor_pos = textureLoad(position_texture, vec2<u32>(u32(nx), u32(ny)), 0);

// //                     if neighbor_pos.a > 0.0 {
// //                         let neighbor_world = edl_params.bounds_min + neighbor_pos.xyz * (edl_params.bounds_max - edl_params.bounds_min);
// //                         let neighbor_raw_depth = length(neighbor_world - depth_camera);

// //                         // Normalize and apply curve
// //                         let neighbor_normalized = clamp((neighbor_raw_depth - depth_range.x) / (depth_range.y - depth_range.x), 0.0, 1.0);
// //                         let neighbor_curved = apply_depth_curve(neighbor_normalized);

// //                         total_normalized_depth += neighbor_curved;
// //                         count += 1u;
// //                     }
// //                 }
// //             }
// //         }

// //         if count > 0u {
// //             final_normalized_depth = total_normalized_depth / f32(count);
// //         }
// //     }

// //     // Ensure depth is in [0,1] range
// //     final_normalized_depth = clamp(final_normalized_depth, 0.0, 1.0);

// //     // Store color in RGB, normalized depth in alpha
// //     textureStore(color_output, coords, vec4<f32>(color_sample.rgb, final_normalized_depth));
// // }

// // === ADAPTIVE RADIAL DEPTH CONTROL CONSTANTS ===
// // Get high precision in a focus radius, compress the rest

// // Fixed Y height for the depth camera
// const DEPTH_CAMERA_Y: f32 = 10.0;

// // Focus radius - objects within this distance get high precision
// const FOCUS_RADIUS: f32 = 50.0;

// // How much of the [0,1] range to dedicate to the focus radius
// // 0.7 = 70% of precision for focus radius, 30% for everything beyond
// const FOCUS_PRECISION_RATIO: f32 = 0.7;

// // Transition smoothness between focus and far zones
// // Higher = sharper transition, Lower = smoother blend
// const TRANSITION_SMOOTHNESS: f32 = 2.0;

// // Curve type for the focus zone (fine details)
// // 0: Linear, 1: Slight curve for even finer close detail
// const FOCUS_CURVE: i32 = 0;

// // Curve type for the far zone (compression)
// // 0: Linear, 1: Logarithmic (more detail for closer-far objects)
// const FAR_CURVE: i32 = 1;

// // ============================================

// @group(0) @binding(0) var position_texture: texture_2d<f32>;
// @group(0) @binding(1) var color_input: texture_2d<f32>;
// @group(0) @binding(2) var color_output: texture_storage_2d<rgba32float, write>;

// struct EDLUniforms {
//     view_matrix: mat4x4<f32>,
//     camera_pos: vec3<f32>,
//     _padding1: f32,
//     bounds_min: vec3<f32>,
//     _padding2: f32,
//     bounds_max: vec3<f32>,
//     _padding3: f32,
// }

// @group(0) @binding(3) var<uniform> edl_params: EDLUniforms;

// // Calculate max distance in scene for far zone normalization
// fn calculate_max_scene_distance(depth_camera: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> f32 {
//     let corners = array<vec3<f32>, 8>(
//         vec3<f32>(bounds_min.x, bounds_min.y, bounds_min.z),
//         vec3<f32>(bounds_max.x, bounds_min.y, bounds_min.z),
//         vec3<f32>(bounds_min.x, bounds_max.y, bounds_min.z),
//         vec3<f32>(bounds_max.x, bounds_max.y, bounds_min.z),
//         vec3<f32>(bounds_min.x, bounds_min.y, bounds_max.z),
//         vec3<f32>(bounds_max.x, bounds_min.y, bounds_max.z),
//         vec3<f32>(bounds_min.x, bounds_max.y, bounds_max.z),
//         vec3<f32>(bounds_max.x, bounds_max.y, bounds_max.z)
//     );

//     var max_dist = length(corners[0] - depth_camera);
//     for (var i = 1; i < 8; i++) {
//         let dist = length(corners[i] - depth_camera);
//         max_dist = max(max_dist, dist);
//     }
//     return max_dist;
// }

// // Apply curve to focus zone
// fn apply_focus_curve(t: f32) -> f32 {
//     switch FOCUS_CURVE {
//         case 1: {
//             // Slight power curve for even finer close detail
//             return pow(t, 1.5);
//         }
//         default: {
//             return t; // Linear
//         }
//     }
// }

// // Apply curve to far zone
// fn apply_far_curve(t: f32) -> f32 {
//     switch FAR_CURVE {
//         case 1: {
//             // Logarithmic - gives more detail to closer-far objects
//             return log(1.0 + t * 9.0) / log(10.0);
//         }
//         default: {
//             return t; // Linear
//         }
//     }
// }

// // Adaptive depth remapping - high precision near camera, compressed far
// fn remap_adaptive_depth(raw_depth: f32, max_scene_distance: f32) -> f32 {
//     if raw_depth <= FOCUS_RADIUS {
//         // High precision zone: map [0, FOCUS_RADIUS] to [0, FOCUS_PRECISION_RATIO]
//         let focus_t = raw_depth / FOCUS_RADIUS;
//         let curved_focus_t = apply_focus_curve(focus_t);
//         return curved_focus_t * FOCUS_PRECISION_RATIO;
//     } else {
//         // Compressed far zone: map [FOCUS_RADIUS, max_distance] to [FOCUS_PRECISION_RATIO, 1.0]
//         let far_range = max_scene_distance - FOCUS_RADIUS;
//         let far_t = clamp((raw_depth - FOCUS_RADIUS) / far_range, 0.0, 1.0);
//         let curved_far_t = apply_far_curve(far_t);

//         // Smooth transition to avoid sharp edge at focus radius boundary
//         let transition_factor = 1.0 - exp(-TRANSITION_SMOOTHNESS * far_t);
//         let blended_far_t = mix(0.0, curved_far_t, transition_factor);

//         return FOCUS_PRECISION_RATIO + blended_far_t * (1.0 - FOCUS_PRECISION_RATIO);
//     }
// }

// @compute @workgroup_size(8, 8, 1)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let coords = global_id.xy;
//     let texture_size = textureDimensions(color_input);

//     if coords.x >= texture_size.x || coords.y >= texture_size.y {
//         return;
//     }

//     // Create depth camera with fixed Y height
//     var depth_camera = vec3<f32>(edl_params.camera_pos.x, DEPTH_CAMERA_Y, edl_params.camera_pos.z);

//     let position_sample = textureLoad(position_texture, coords, 0);
//     let color_sample = textureLoad(color_input, coords, 0);

//     // Calculate max scene distance for far zone normalization
//     let max_distance = calculate_max_scene_distance(depth_camera, edl_params.bounds_min, edl_params.bounds_max);

//     var final_normalized_depth = 0.0;

//     // If we have a valid point at this pixel
//     if position_sample.a > 0.0 {
//         let world_pos = edl_params.bounds_min + position_sample.xyz * (edl_params.bounds_max - edl_params.bounds_min);
//         let raw_depth = length(world_pos - depth_camera);

//         final_normalized_depth = remap_adaptive_depth(raw_depth, max_distance);
//     } else {
//         // Interpolate depth from nearby valid points
//         var total_normalized_depth = 0.0;
//         var count = 0u;

//         for (var dx = -3; dx <= 3; dx++) {
//             for (var dy = -3; dy <= 3; dy++) {
//                 if dx == 0 && dy == 0 { continue; }

//                 let nx = i32(coords.x) + dx;
//                 let ny = i32(coords.y) + dy;

//                 if nx >= 0 && nx < i32(texture_size.x) && ny >= 0 && ny < i32(texture_size.y) {
//                     let neighbor_pos = textureLoad(position_texture, vec2<u32>(u32(nx), u32(ny)), 0);

//                     if neighbor_pos.a > 0.0 {
//                         let neighbor_world = edl_params.bounds_min + neighbor_pos.xyz * (edl_params.bounds_max - edl_params.bounds_min);
//                         let neighbor_raw_depth = length(neighbor_world - depth_camera);
//                         let neighbor_normalized_depth = remap_adaptive_depth(neighbor_raw_depth, max_distance);

//                         total_normalized_depth += neighbor_normalized_depth;
//                         count += 1u;
//                     }
//                 }
//             }
//         }

//         if count > 0u {
//             final_normalized_depth = total_normalized_depth / f32(count);
//         }
//     }

//     // Ensure depth is in [0,1] range
//     final_normalized_depth = clamp(final_normalized_depth, 0.0, 1.0);

//     // Store color in RGB, normalized depth in alpha
//     textureStore(color_output, coords, vec4<f32>(color_sample.rgb, final_normalized_depth));
// }


// // === SIMPLE NON-LINEAR DEPTH REMAPPING ===
// // Power curve to expand small/medium depth differences

// // Fixed Y height for the depth camera
// const DEPTH_CAMERA_Y: f32 = 100.0;

// // Power curve strength - THIS IS THE MAIN CONTROL
// // 0.5 = Square root (expands small differences a lot)
// // 0.3 = Cube root (expands small differences even more)
// // 0.7 = Less expansion but still non-linear
// const POWER_CURVE: f32 = 0.3;

// // ============================================

// @group(0) @binding(0) var position_texture: texture_2d<f32>;
// @group(0) @binding(1) var color_input: texture_2d<f32>;
// @group(0) @binding(2) var color_output: texture_storage_2d<rgba32float, write>;

// struct EDLUniforms {
//     view_matrix: mat4x4<f32>,
//     camera_pos: vec3<f32>,
//     _padding1: f32,
//     bounds_min: vec3<f32>,
//     _padding2: f32,
//     bounds_max: vec3<f32>,
//     _padding3: f32,
// }

// @group(0) @binding(3) var<uniform> edl_params: EDLUniforms;

// // Calculate scene depth bounds
// fn calculate_depth_bounds(depth_camera: vec3<f32>, bounds_min: vec3<f32>, bounds_max: vec3<f32>) -> vec2<f32> {
//     let corners = array<vec3<f32>, 8>(
//         vec3<f32>(bounds_min.x, bounds_min.y, bounds_min.z),
//         vec3<f32>(bounds_max.x, bounds_min.y, bounds_min.z),
//         vec3<f32>(bounds_min.x, bounds_max.y, bounds_min.z),
//         vec3<f32>(bounds_max.x, bounds_max.y, bounds_min.z),
//         vec3<f32>(bounds_min.x, bounds_min.y, bounds_max.z),
//         vec3<f32>(bounds_max.x, bounds_min.y, bounds_max.z),
//         vec3<f32>(bounds_min.x, bounds_max.y, bounds_max.z),
//         vec3<f32>(bounds_max.x, bounds_max.y, bounds_max.z)
//     );

//     var min_dist = length(corners[0] - depth_camera);
//     var max_dist = min_dist;

//     for (var i = 1; i < 8; i++) {
//         let dist = length(corners[i] - depth_camera);
//         min_dist = min(min_dist, dist);
//         max_dist = max(max_dist, dist);
//     }

//     return vec2<f32>(min_dist, max_dist);
// }

// @compute @workgroup_size(8, 8, 1)
// fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
//     let coords = global_id.xy;
//     let texture_size = textureDimensions(color_input);

//     if coords.x >= texture_size.x || coords.y >= texture_size.y {
//         return;
//     }

//     // Create depth camera with fixed Y height
//     var depth_camera = vec3<f32>(edl_params.camera_pos.x, DEPTH_CAMERA_Y, edl_params.camera_pos.z);

//     let position_sample = textureLoad(position_texture, coords, 0);
//     let color_sample = textureLoad(color_input, coords, 0);

//     // Calculate depth range for normalization
//     let depth_range = calculate_depth_bounds(depth_camera, edl_params.bounds_min, edl_params.bounds_max);

//     var final_depth = 0.0;

//     // If we have a valid point at this pixel
//     if position_sample.a > 0.0 {
//         let world_pos = edl_params.bounds_min + position_sample.xyz * (edl_params.bounds_max - edl_params.bounds_min);
//         let raw_depth = length(world_pos - depth_camera);

//         // Normalize to [0,1]
//         let normalized = clamp((raw_depth - depth_range.x) / (depth_range.y - depth_range.x), 0.0, 1.0);

//         // Apply power curve - THIS IS THE MAGIC LINE
//         final_depth = pow(normalized, POWER_CURVE);
//     } else {
//         // Interpolate depth from nearby valid points
//         var total_depth = 0.0;
//         var count = 0u;

//         for (var dx = -3; dx <= 3; dx++) {
//             for (var dy = -3; dy <= 3; dy++) {
//                 if dx == 0 && dy == 0 { continue; }

//                 let nx = i32(coords.x) + dx;
//                 let ny = i32(coords.y) + dy;

//                 if nx >= 0 && nx < i32(texture_size.x) && ny >= 0 && ny < i32(texture_size.y) {
//                     let neighbor_pos = textureLoad(position_texture, vec2<u32>(u32(nx), u32(ny)), 0);

//                     if neighbor_pos.a > 0.0 {
//                         let neighbor_world = edl_params.bounds_min + neighbor_pos.xyz * (edl_params.bounds_max - edl_params.bounds_min);
//                         let neighbor_raw_depth = length(neighbor_world - depth_camera);

//                         let neighbor_normalized = clamp((neighbor_raw_depth - depth_range.x) / (depth_range.y - depth_range.x), 0.0, 1.0);
//                         let neighbor_curved = pow(neighbor_normalized, POWER_CURVE);

//                         total_depth += neighbor_curved;
//                         count += 1u;
//                     }
//                 }
//             }
//         }

//         if count > 0u {
//             final_depth = total_depth / f32(count);
//         }
//     }

//     // Ensure depth is in [0,1] range
//     final_depth = clamp(final_depth, 0.0, 1.0);

//     // Store color in RGB, depth in alpha
//     textureStore(color_output, coords, vec4<f32>(color_sample.rgb, final_depth));
// }
