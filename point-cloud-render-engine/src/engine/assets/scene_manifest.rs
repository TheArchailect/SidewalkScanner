use crate::constants::texture::TEXTURE_SIZE;
use crate::engine::assets::asset_definitions::AssetDefinition;
use crate::engine::assets::asset_definitions::AtlasConfig;
use crate::engine::assets::bounds::BoundsData;
use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::texture_files::{AssetTextureFiles, TerrainTextureFiles};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use serde::{Deserialize, Serialize};

/// Terrain point cloud data with texture file references and metadata.
/// Contains the core point cloud information that was previously in bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainData {
    pub texture_files: TerrainTextureFiles,
    pub bounds: BoundsData,
    pub point_count: usize,
    pub has_colour: bool,
}

/// Asset atlas containing texture references and individual asset definitions.
/// Enables placement of 3D objects within the point cloud scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAtlasData {
    pub texture_files: AssetTextureFiles,
    pub assets: Vec<AssetDefinition>,
    pub atlas_config: AtlasConfig,
}

/// Complete scene manifest as a Bevy asset. Mirrors JSON structure exactly.
/// Contains both terrain point cloud data and optional asset atlas information.
#[derive(Asset, Debug, Clone, Serialize, Deserialize, TypePath, Resource, ExtractResource)]
pub struct SceneManifest {
    pub terrain: TerrainData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_atlas: Option<AssetAtlasData>,
    pub scene_bounds: BoundsData,
}

impl SceneManifest {
    /// Get terrain bounds for camera positioning and frustum culling.
    /// Returns reference to avoid unnecessary cloning in hot paths.
    pub fn terrain_bounds(&self) -> &BoundsData {
        &self.terrain.bounds
    }

    /// Extract legacy PointCloudBounds for compatibility with existing systems.
    /// Use sparingly - prefer direct manifest access for new code.
    pub fn to_point_cloud_bounds(&self) -> PointCloudBounds {
        PointCloudBounds {
            bounds: self.terrain.bounds.clone(),
            total_points: self.terrain.point_count,
            loaded_points: self.terrain.point_count,
            texture_size: TEXTURE_SIZE as u32, // Standard unified texture resolution.
            sampling_ratio: 1.0,               // Full resolution for new manifests.
            utilisation_percent: 100.0,
            has_colour: self.terrain.has_colour,
            colour_points: if self.terrain.has_colour {
                self.terrain.point_count
            } else {
                0
            },
            road_points: 0, // Requires post-processing classification analysis.
        }
    }
}
