use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// DDS texture file paths for terrain point cloud data.
/// Replaces hardcoded texture path construction in loading systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTextureFiles {
    pub position: String,
    pub colour_class: String,
    pub spatial_index: String,
    pub heightmap: String,
}

/// DDS texture file paths for asset atlas data.
/// Asset atlas uses separate textures from terrain for independent resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTextureFiles {
    pub position: String,
    pub colour_class: String,
}
