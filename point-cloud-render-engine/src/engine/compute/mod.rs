//! GPU compute pipeline modules for point cloud processing.
//!
//! Implements procedural texture modification and depth computation for the rendering pipeline.
//! All compute operations are non-destructive, writing results to separate output textures.
//!
//! ## Pipeline Architecture
//!
//! ### Phase 1: Classification Compute (`compute_classification`)
//! Procedurally modifies point classifications using user-defined polygon masks and spatial filtering.
//!
//! **Input textures:**
//! - `colour_class_texture`: RGB + classification (alpha channel)
//! - `position_texture`: Normalised XYZ + connectivity class ID
//! - `spatial_index_texture`: Morton codes for spatial acceleration
//!
//! **Output:**
//! - `result_texture`: Reclassified points with updated classification values
//!
//! **Features:**
//! - Polygon-based point selection with spatial AABB/Morton filtering
//! - Hide/reclassify operations with mask-based ignore rules
//! - Hover/selection highlighting for user feedback
//! - Hidden points marked with classification `254`
//!
//! ### Phase 2: Depth Compute (`edl_compute_depth`)
//! Calculates depth values from classified points for eye-dome lighting (EDL) shading.
//!
//! **Input textures:**
//! - `position_texture`: Point positions in world space
//! - `result_texture`: Classified RGB + depth
//!
//! **Output:**
//! - `depth_texture`: R32F depth buffer for EDL processing
//!
//! ## WGSL Shader Bindings
//!
//! Classification compute shader expects:
//! ```wgsl
//! @group(0) @binding(0) var colour_texture: texture_2d<f32>;      // Input: RGB + class
//! @group(0) @binding(1) var position_texture: texture_2d<f32>;    // Input: XYZ + ID
//! @group(0) @binding(2) var spatial_index_texture: texture_2d<f32>; // Input: Morton codes
//! @group(0) @binding(3) var result_texture: texture_storage_2d<rgba32float, write>; // Output
//! @group(0) @binding(4) var<uniform> polygon_data: PolygonUniform; // Polygon definitions
//! @group(0) @binding(5) var<uniform> terrain_bounds: TerrainBounds; // World bounds
//! ```
//!
//! Depth compute shader expects:
//! ```wgsl
//! @group(0) @binding(0) var position_texture: texture_2d<f32>;    // Input: world positions
//! @group(0) @binding(1) var colour_texture: texture_2d<f32>;      // Input: classified RGB
//! @group(0) @binding(2) var depth_output: texture_storage_2d<r32float, write>; // Output
//! @group(0) @binding(3) var<uniform> camera_data: CameraUniforms;  // View/projection
//! ```
//!
//! ## Performance Considerations
//!
//! - Spatial filtering significantly reduces cost on large point clouds
//! - Morton filtering provides granular culling but higher compute overhead
//! - AABB mode offers faster broad-phase rejection for simple polygons
//! - Compute shaders only execute when state changes (trigger systems)

/// Procedural point classification using polygon masks and spatial filtering.
///
/// Non-destructive reclassification with hide/show operations and user feedback highlighting.
pub mod compute_classification;

/// Depth buffer computation for eye-dome lighting (EDL) shader effects.
///
/// Generates camera-space depth from world positions for post-processing shading.
pub mod edl_compute_depth;
