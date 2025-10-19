//! Rendering systems for point cloud and instanced asset pipelines.
//!
//! Implements custom render pipelines, post-processing effects, and resource
//! extraction for GPU-side point cloud rendering with asset instancing support.

/// Eye-dome lighting (EDL) post-processing for depth-based shading enhancement.
///
/// Fullscreen shader pass that enhances depth perception using neighbouring pixel depth comparison.
pub mod edl_post_processing;

/// Resource extraction systems transferring main world state to render world.
///
/// Synchronises camera data, bounds, application state, and scene manifests each frame.
pub mod extraction;

/// Instanced asset rendering pipeline for GPU-efficient object placement.
///
/// Renders multiple asset instances using per-instance vertex buffers and atlas texture sampling.
pub mod instanced_render_plugin;

/// Custom point cloud render pipeline with specialised material bindings.
///
/// GPU-side vertex expansion pipeline converting indexed vertices to screen-aligned quads.
pub mod pipeline;
