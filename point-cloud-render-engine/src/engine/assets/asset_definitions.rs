use crate::engine::assets::bounds::BoundsData;
use bevy::prelude::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
/// Individual asset definition with atlas position and local bounds.
/// Defines placement and rendering data for objects in the scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDefinition {
    pub name: String,
    pub atlas_position: [u32; 2], // Grid position in 2048x2048 atlas.
    pub uv_bounds: UVBounds,      // Normalized texture coordinates [0.0, 1.0].
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

impl AssetDefinition {
    /// Calculate center point in local coordinates for transform calculations.
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            ((self.local_bounds.max_x + self.local_bounds.min_x) * 0.5) as f32,
            ((self.local_bounds.max_y + self.local_bounds.min_y) * 0.5) as f32,
            ((self.local_bounds.max_z + self.local_bounds.min_z) * 0.5) as f32,
        )
    }

    /// Calculate asset dimensions for LOD selection and culling decisions.
    pub fn size(&self) -> Vec3 {
        Vec3::new(
            (self.local_bounds.max_x - self.local_bounds.min_x) as f32,
            (self.local_bounds.max_y - self.local_bounds.min_y) as f32,
            (self.local_bounds.max_z - self.local_bounds.min_z) as f32,
        )
    }

    /// Get UV coordinates as Vec4 for efficient shader uniform uploads.
    /// Packs min and max UV coordinates into single vector for GPU transfer.
    pub fn uv_bounds_vec4(&self) -> Vec4 {
        Vec4::new(
            self.uv_bounds.uv_min[0],
            self.uv_bounds.uv_min[1],
            self.uv_bounds.uv_max[0],
            self.uv_bounds.uv_max[1],
        )
    }
}
