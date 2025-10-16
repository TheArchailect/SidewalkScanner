use crate::engine::assets::bounds::BoundsData;
use serde::{Deserialize, Serialize};
/// Individual asset definition with atlas position and local bounds.
/// Defines placement and rendering data for objects in the scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDefinition {
    pub name: String,
    pub atlas_position: [u32; 2], // Grid position in 2048x2048 atlas.
    pub uv_bounds: UVBounds,      // Normalised texture coordinates [0.0, 1.0].
    pub local_bounds: BoundsData, // 3D bounds in asset-local space.
    pub point_count: usize,
}

/// UV texture coordinate bounds for atlas tile sampling.
/// Defines the region within the atlas texture for this asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UVBounds {
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

/// Atlas configuration defining texture layout and capacity.
/// Determines how assets are packed into the unified texture atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConfig {
    pub atlas_size: u32, // Total texture dimensions (e.g., 2048).
    pub tile_size: u32,  // Individual asset tile size (e.g., 256).
    pub max_assets: u32, // Maximum assets supported in atlas.
}
