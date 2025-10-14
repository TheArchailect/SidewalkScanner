//! Mesh generation for point cloud rendering primitives.
//!
//! Provides GPU-optimised vertex buffer structures for custom render pipelines
//! that expand indexed vertices into screen-aligned quads in vertex shaders.

/// Point cloud index mesh generation for GPU-side vertex expansion.
///
/// Creates triangle-based geometry where each point expands to a quad via vertex shader.
pub mod point_index_mesh;
