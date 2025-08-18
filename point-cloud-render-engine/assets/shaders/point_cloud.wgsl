// #import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

// struct PointCloudMaterial {
//     params: array<vec4<f32>, 2>,
// }

// struct PolygonClassificationData {
//    polygon_count: u32,
//    total_points: u32,
//    _padding1: u32,
//    _padding2: u32,
//    point_data: array<vec4<f32>, 512>,
//    polygon_info: array<vec4<f32>, 64>,
// }
// @group(2) @binding(5) var<uniform> polygon_classification: PolygonClassificationData;


// @group(2) @binding(0) var position_texture: texture_2d<f32>;
// @group(2) @binding(1) var position_sampler: sampler;
// @group(2) @binding(2) var metadata_texture: texture_2d<f32>;
// @group(2) @binding(3) var metadata_sampler: sampler;
// @group(2) @binding(4) var<uniform> material: PointCloudMaterial;

// struct VertexInput {
//     @location(0) position: vec3<f32>,
// }

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec4<f32>,
// }

// @vertex
// fn vertex(vertex: VertexInput) -> VertexOutput {
//     var out: VertexOutput;

//     let point_index = u32(vertex.position.x);

//     // Convert point index to texture coordinates
//     let texture_size = material.params[0].w;
//     let x_coord = point_index % u32(texture_size);
//     let y_coord = point_index / u32(texture_size);

//     // Sample position from texture using UV
//     let uv = vec2<f32>(
//         (f32(x_coord) + 0.5) / texture_size,
//         (f32(y_coord) + 0.5) / texture_size
//     );
//     let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

//     // Skip invalid points (alpha == 0)
//     if pos_sample.a == 0.0 {
//         out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0); // Move far behind camera
//         out.color = vec4<f32>(0.0);
//         return out;
//     }

//     // Denormalize coordinates
//     let norm_pos = pos_sample.rgb;
//     let min_pos = material.params[0].xyz;
//     let max_pos = material.params[1].xyz;
//     let range = max_pos - min_pos;
//     let world_pos = min_pos + norm_pos * range;

//     // Clamp positions to prevent extreme values
//     let clamped_pos = clamp(world_pos, min_pos - range * 0.1, max_pos + range * 0.1);

//     // Transform to clip space
//     let clip_pos = mesh_position_local_to_clip(mat4x4<f32>(
//         1.0, 0.0, 0.0, 0.0,
//         0.0, 1.0, 0.0, 0.0,
//         0.0, 0.0, 1.0, 0.0,
//         0.0, 0.0, 0.0, 1.0
//     ), vec4<f32>(clamped_pos, 1.0));


//     // Ensure w component is valid
//     if clip_pos.w <= 0.0 {
//         out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
//         out.color = vec4<f32>(0.0);
//         return out;
//     }

//     out.clip_position = clip_pos;

//     // Sample metadata using exact pixel coordinates
//     let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
//     let meta_sample = textureLoad(metadata_texture, tex_coord, 0);
//     let classification = u32(meta_sample.r * 255.0);
//     let intensity = meta_sample.g;

//     var final_classification = classification;
//     for (var i = 0u; i < polygon_classification.polygon_count; i = i + 1u) {
//         if point_in_polygon(world_pos.x, world_pos.z, i) {
//             final_classification = u32(polygon_classification.polygon_info[i].z);
//             break;
//         }
//     }

//     // Convert classification to color
//     out.color = classification_to_color(final_classification, intensity);

//     return out;
// }

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     return in.color;
// }

// fn classification_to_color(classification: u32, intensity: f32) -> vec4<f32> {
//     let c = classification & 255u;
//     let hash1 = (c * 73u) % 255u;
//     let hash2 = (c * 151u + 17u) % 255u;
//     let hash3 = (c * 211u + 37u) % 255u;

//     let r = f32(hash1) / 255.0;
//     let g = f32(hash2) / 255.0;
//     let b = f32(hash3) / 255.0;

//     let brightness = 0.7 + intensity * 0.3;
//     return vec4<f32>(r * brightness, g * brightness, b * brightness, 1.0);
// }

// // fn point_in_polygon(point_x: f32, point_z: f32, polygon_idx: u32) -> bool {
// //     let poly = polygon_classification.polygons[polygon_idx];
// //     let point_count = u32(poly.metadata.y);

// //     var inside = false;
// //     var j = point_count - 1u;

// //     for (var i = 0u; i < point_count; i = i + 1u) {
// //         var xi: f32;
// //         var zi: f32;
// //         var xj: f32;
// //         var zj: f32;

// //         // Get current point
// //         if i == 0u {
// //             xi = poly.coords1.x; zi = poly.coords1.y;
// //         } else if i == 1u {
// //             xi = poly.coords1.z; zi = poly.coords1.w;
// //         } else if i == 2u {
// //             xi = poly.coords2.x; zi = poly.coords2.y;
// //         } else {
// //             xi = poly.coords2.z; zi = poly.coords2.w;
// //         }

// //         // Get previous point
// //         if j == 0u {
// //             xj = poly.coords1.x; zj = poly.coords1.y;
// //         } else if j == 1u {
// //             xj = poly.coords1.z; zj = poly.coords1.w;
// //         } else if j == 2u {
// //             xj = poly.coords2.x; zj = poly.coords2.y;
// //         } else {
// //             xj = poly.coords2.z; zj = poly.coords2.w;
// //         }

// //         if ((zi > point_z) != (zj > point_z)) &&
// //            (point_x < (xj - xi) * (point_z - zi) / (zj - zi) + xi) {
// //             inside = !inside;
// //         }
// //         j = i;
// //     }
// //     return inside;
// // }

// fn point_in_polygon(point_x: f32, point_z: f32, polygon_idx: u32) -> bool {
//     let info = polygon_classification.polygon_info[polygon_idx];
//     let start_idx = u32(info.x);
//     let point_count = u32(info.y);

//     var inside = false;
//     var j = point_count - 1u;

//     for (var i = 0u; i < point_count; i = i + 1u) {
//         let curr_pt = polygon_classification.point_data[start_idx + i].xy;
//         let prev_pt = polygon_classification.point_data[start_idx + j].xy;

//         if ((curr_pt.y > point_z) != (prev_pt.y > point_z)) &&
//            (point_x < (prev_pt.x - curr_pt.x) * (point_z - curr_pt.y) / (prev_pt.y - curr_pt.y) + curr_pt.x) {
//             inside = !inside;
//         }
//         j = i;
//     }
//     return inside;
// }

// // Point cloud vertex/fragment shader with unified texture support
// #import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

// struct PointCloudMaterial {
//     params: array<vec4<f32>, 2>,
// }

// struct PolygonClassificationData {
//    polygon_count: u32,
//    total_points: u32,
//    _padding1: u32,
//    _padding2: u32,
//    point_data: array<vec4<f32>, 512>,
//    polygon_info: array<vec4<f32>, 64>,
// }

// @group(2) @binding(0) var position_texture: texture_2d<f32>;      // RGBA32F: XYZ + validity
// @group(2) @binding(1) var position_sampler: sampler;
// @group(2) @binding(2) var colour_class_texture: texture_2d<f32>;  // RGBA32F: RGB + classification
// @group(2) @binding(3) var colour_class_sampler: sampler;
// @group(2) @binding(4) var<uniform> material: PointCloudMaterial;
// @group(2) @binding(5) var<uniform> polygon_classification: PolygonClassificationData;

// struct VertexInput {
//     @location(0) position: vec3<f32>,
// }

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec4<f32>,
// }

// @vertex
// fn vertex(vertex: VertexInput) -> VertexOutput {
//     var out: VertexOutput;

//     let point_index = u32(vertex.position.x);

//     // Convert point index to texture coordinates
//     let texture_size = material.params[0].w;
//     let x_coord = point_index % u32(texture_size);
//     let y_coord = point_index / u32(texture_size);

//     // Sample position from RGBA32F texture
//     let uv = vec2<f32>(
//         (f32(x_coord) + 0.5) / texture_size,
//         (f32(y_coord) + 0.5) / texture_size
//     );
//     let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

//     // Skip invalid points (alpha == 0)
//     if pos_sample.a == 0.0 {
//         out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
//         out.color = vec4<f32>(0.0);
//         return out;
//     }

//     // Denormalise coordinates to world space
//     let norm_pos = pos_sample.rgb;
//     let min_pos = material.params[0].xyz;
//     let max_pos = material.params[1].xyz;
//     let range = max_pos - min_pos;
//     let world_pos = min_pos + norm_pos * range;

//     // Clamp positions to prevent extreme values
//     let clamped_pos = clamp(world_pos, min_pos - range * 0.1, max_pos + range * 0.1);

//     // Transform to clip space
//     let clip_pos = mesh_position_local_to_clip(mat4x4<f32>(
//         1.0, 0.0, 0.0, 0.0,
//         0.0, 1.0, 0.0, 0.0,
//         0.0, 0.0, 1.0, 0.0,
//         0.0, 0.0, 0.0, 1.0
//     ), vec4<f32>(clamped_pos, 1.0));

//     // Ensure valid clip space coordinates
//     if clip_pos.w <= 0.0 {
//         out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
//         out.color = vec4<f32>(0.0);
//         return out;
//     }

//     out.clip_position = clip_pos;

//     // Sample colour and classification from unified texture
//     let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
//     let colour_class_sample = textureLoad(colour_class_texture, tex_coord, 0);

//     // Extract RGB colour and classification
//     let rgb_colour = colour_class_sample.rgb;
//     let classification = u32(colour_class_sample.a * 255.0);

//     // Check for polygon classification override
//     var final_classification = classification;
//     for (var i = 0u; i < polygon_classification.polygon_count; i = i + 1u) {
//         if point_in_polygon(world_pos.x, world_pos.z, i) {
//             final_classification = u32(polygon_classification.polygon_info[i].z);
//             break;
//         }
//     }

//     // Use actual colour data if available, otherwise generate from classification
//     if length(rgb_colour) > 0.1 {
//         // Use real colour data with classification-based brightness adjustment
//         let brightness = classification_brightness(final_classification);
//         out.color = vec4<f32>(rgb_colour * brightness, 1.0);
//     } else {
//         // Fallback to classification-based colour generation
//         out.color = classification_to_color(final_classification);
//     }

//     return out;
// }

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     return in.color;
// }

// /// Generate brightness multiplier based on classification
// fn classification_brightness(classification: u32) -> f32 {
//     // Road surfaces brighter, vegetation darker, buildings medium
//     switch classification {
//         case 2u: { return 1.2; }   // Roads
//         case 10u: { return 1.2; }  // Roads
//         case 11u: { return 1.2; }  // Roads
//         case 12u: { return 1.2; }  // Roads
//         case 3u: { return 0.8; }   // Vegetation
//         case 4u: { return 0.8; }   // Vegetation
//         case 5u: { return 0.8; }   // Vegetation
//         case 6u: { return 1.1; }   // Buildings
//         default: { return 1.0; }
//     }
// }

// /// Generate colour from classification when RGB data unavailable
// fn classification_to_color(classification: u32) -> vec4<f32> {
//     let c = classification & 255u;
//     let hash1 = (c * 73u) % 255u;
//     let hash2 = (c * 151u + 17u) % 255u;
//     let hash3 = (c * 211u + 37u) % 255u;

//     let r = f32(hash1) / 255.0;
//     let g = f32(hash2) / 255.0;
//     let b = f32(hash3) / 255.0;

//     return vec4<f32>(r, g, b, 1.0);
// }

// /// Point-in-polygon test using polygon classification data
// fn point_in_polygon(point_x: f32, point_z: f32, polygon_idx: u32) -> bool {
//     let info = polygon_classification.polygon_info[polygon_idx];
//     let start_idx = u32(info.x);
//     let point_count = u32(info.y);

//     var inside = false;
//     var j = point_count - 1u;

//     for (var i = 0u; i < point_count; i = i + 1u) {
//         let curr_pt = polygon_classification.point_data[start_idx + i].xy;
//         let prev_pt = polygon_classification.point_data[start_idx + j].xy;

//         if ((curr_pt.y > point_z) != (prev_pt.y > point_z)) &&
//            (point_x < (prev_pt.x - curr_pt.x) * (point_z - curr_pt.y) / (prev_pt.y - curr_pt.y) + curr_pt.x) {
//             inside = !inside;
//         }
//         j = i;
//     }
//     return inside;
// }

// Point cloud vertex/fragment shader with unified texture support
#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct PointCloudMaterial {
    params: array<vec4<f32>, 2>,
}

struct PolygonClassificationData {
   polygon_count: u32,
   total_points: u32,
   render_mode: u32,
   _padding: u32,
   point_data: array<vec4<f32>, 512>,
   polygon_info: array<vec4<f32>, 64>,
}

@group(2) @binding(0) var position_texture: texture_2d<f32>;      // RGBA32F: XYZ + validity
@group(2) @binding(1) var position_sampler: sampler;
@group(2) @binding(2) var colour_class_texture: texture_2d<f32>;  // RGBA32F: RGB + classification
@group(2) @binding(3) var colour_class_sampler: sampler;
@group(2) @binding(4) var<uniform> material: PointCloudMaterial;
@group(2) @binding(5) var<uniform> polygon_classification: PolygonClassificationData;

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

    // Sample position from RGBA32F texture
    let uv = vec2<f32>(
        (f32(x_coord) + 0.5) / texture_size,
        (f32(y_coord) + 0.5) / texture_size
    );
    let pos_sample = textureSampleLevel(position_texture, position_sampler, uv, 0.0);

    // Skip invalid points (alpha == 0)
    if pos_sample.a == 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    // Denormalise coordinates to world space
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

    // Ensure valid clip space coordinates
    if clip_pos.w <= 0.0 {
        out.clip_position = vec4<f32>(0.0, 0.0, -10.0, 1.0);
        out.color = vec4<f32>(0.0);
        return out;
    }

    out.clip_position = clip_pos;

    // Sample colour and classification from unified texture
    let tex_coord = vec2<i32>(i32(x_coord), i32(y_coord));
    let colour_class_sample = textureLoad(colour_class_texture, tex_coord, 0);

    // Extract RGB colour and classification
    let rgb_colour = colour_class_sample.rgb;
    let original_classification = u32(colour_class_sample.a * 255.0);

    // Check for polygon classification override
    var modified_classification = original_classification;
    for (var i = 0u; i < polygon_classification.polygon_count; i = i + 1u) {
        if point_in_polygon(world_pos.x, world_pos.z, i) {
            modified_classification = u32(polygon_classification.polygon_info[i].z);
            break;
        }
    }

    // Render based on mode
    switch polygon_classification.render_mode {
        case 0u: { // Original classification
            out.color = classification_to_color(original_classification);
        }
        case 1u: { // Modified classification
            out.color = classification_to_color(modified_classification);
        }
        case 2u: { // RGB colour
            if length(rgb_colour) > 0.1 {
                let brightness = classification_brightness(modified_classification);
                out.color = vec4<f32>(rgb_colour * brightness, 1.0);
            } else {
                out.color = classification_to_color(modified_classification);
            }
        }
        default: {
            out.color = vec4<f32>(rgb_colour, 1.0);
        }
    }

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

/// Generate brightness multiplier based on classification
fn classification_brightness(classification: u32) -> f32 {
    // Road surfaces brighter, vegetation darker, buildings medium
    switch classification {
        case 2u: { return 1.2; }   // Roads
        case 10u: { return 1.2; }  // Roads
        case 11u: { return 1.2; }  // Roads
        case 12u: { return 1.2; }  // Roads
        case 3u: { return 0.8; }   // Vegetation
        case 4u: { return 0.8; }   // Vegetation
        case 5u: { return 0.8; }   // Vegetation
        case 6u: { return 1.1; }   // Buildings
        default: { return 1.0; }
    }
}

/// Generate colour from classification when RGB data unavailable
fn classification_to_color(classification: u32) -> vec4<f32> {
    let c = classification & 255u;
    let hash1 = (c * 73u) % 255u;
    let hash2 = (c * 151u + 17u) % 255u;
    let hash3 = (c * 211u + 37u) % 255u;

    let r = f32(hash1) / 255.0;
    let g = f32(hash2) / 255.0;
    let b = f32(hash3) / 255.0;

    return vec4<f32>(r, g, b, 1.0);
}

/// Point-in-polygon test using polygon classification data
fn point_in_polygon(point_x: f32, point_z: f32, polygon_idx: u32) -> bool {
    let info = polygon_classification.polygon_info[polygon_idx];
    let start_idx = u32(info.x);
    let point_count = u32(info.y);

    var inside = false;
    var j = point_count - 1u;

    for (var i = 0u; i < point_count; i = i + 1u) {
        let curr_pt = polygon_classification.point_data[start_idx + i].xy;
        let prev_pt = polygon_classification.point_data[start_idx + j].xy;

        if ((curr_pt.y > point_z) != (prev_pt.y > point_z)) &&
           (point_x < (prev_pt.x - curr_pt.x) * (point_z - curr_pt.y) / (prev_pt.y - curr_pt.y) + curr_pt.x) {
            inside = !inside;
        }
        j = i;
    }
    return inside;
}
