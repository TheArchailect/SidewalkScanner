//! Asset management for point cloud terrain and atlas data.
//!
//! Handles scene manifests, texture file references, spatial bounds,
//! and asset definitions for GPU-optimised rendering.

/// Individual asset definitions with atlas positioning and local bounds.
pub mod asset_definitions;

/// Spatial bounds data structures for terrain and asset coordinate systems.
pub mod bounds;

/// Point cloud texture handles and scene metadata for the rendering pipeline.
pub mod point_cloud_assets;

/// Scene manifest containing terrain data and optional asset atlas information.
pub mod scene_manifest;

/// DDS texture file path structures for terrain and asset atlas data.
pub mod texture_files;
