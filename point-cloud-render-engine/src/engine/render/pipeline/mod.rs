//! Custom render pipeline for point cloud vertex expansion and shading.
//!
//! Implements a specialised mesh pipeline that integrates with Bevy's render graph,
//! using GPU-side vertex expansion to convert indexed vertices into screen-aligned quads.
//!
//! ## Pipeline Architecture
//!
//! The point cloud pipeline extends Bevy's `MeshPipeline` with custom material bindings:
//!
//! - **@group(0)**: View uniforms (camera, projection) - automatic via `SetMeshViewBindGroup`
//! - **@group(1)**: Mesh transforms - automatic via `SetMeshBindGroup`
//! - **@group(2)**: Point cloud materials - custom bind group with textures and uniforms
//!
//! ## Vertex Expansion
//!
//! Each point is represented by 6 indexed vertices (2 triangles) that expand into
//! screen-aligned quads in the vertex shader. The vertex shader uses `vertex_index / 6`
//! to determine which point to fetch from textures, and `vertex_index % 6` to position
//! corners of the billboard quad.
//!
//! ## Render Phase Integration
//!
//! 1. **Extract**: Transfer camera and bounds from main world to render world
//! 2. **Prepare**: Create material bind groups with texture views and uniforms
//! 3. **Queue**: Specialise pipelines and add entities to render phase
//! 4. **Sort**: Order phase items by camera distance for proper transparency
//! 5. **Render**: Execute draw commands via `PointCloudRenderNode`
//!
//! ## Material Bindings (@group(2))
//!
//! The shader expects the following bindings:
//!
//! ```wgsl
//! @group(2) @binding(0) var position_texture: texture_2d<f32>;
//! @group(2) @binding(1) var position_sampler: sampler;
//! @group(2) @binding(2) var result_texture: texture_2d<f32>;  // Classified RGB+depth
//! @group(2) @binding(3) var result_sampler: sampler;
//! @group(2) @binding(4) var depth_texture: texture_2d<f32>;   // EDL depth buffer
//! @group(2) @binding(5) var depth_sampler: sampler;
//! @group(2) @binding(6) var<uniform> material: PointCloudMaterial;
//!
//! struct PointCloudMaterial {
//!     params: array<vec4<f32>, 3>,  // [0]=min_bounds+size, [1]=max_bounds, [2]=camera_pos
//! }
//! ```
//!
//! ## Depth Testing
//!
//! The pipeline integrates with Bevy's depth buffer using reversed-Z depth testing
//! with `Greater` comparison. Points write to the depth buffer and load existing depth
//! from the opaque pass, allowing proper occlusion with standard meshes.
//!
//! ## Render Graph Position
//!
//! Point clouds render after opaque geometry but before post-processing via
//! `MainOpaquePass → PointCloudRenderLabel → Tonemapping`, allowing depth buffer
//! interaction whilst avoiding transparency issues.
//!
//! ## Performance Considerations
//!
//! - **No frustum culling**: `NoFrustumCulling` component prevents culling of large point clouds
//! - **Batching**: Single draw call per point cloud entity
//! - **GPU expansion**: Vertex shader handles quad generation, reducing CPU overhead
//! - **Texture sampling**: Non-filtering samplers avoid interpolation artefacts
//! - **Depth sorting**: Camera distance sorting ensures correct render order

/// Custom point cloud render pipeline with GPU-side vertex expansion.
///
/// Integrates with Bevy's mesh pipeline system for automatic view/mesh binding
/// whilst providing specialised material bindings for texture-based point data.
pub mod point_cloud_render_pipeline;
