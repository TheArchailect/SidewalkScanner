use crate::engine::assets::asset_definitions::AssetDefinition;
use crate::engine::assets::asset_definitions::AtlasConfig;
use crate::engine::assets::bounds::BoundsData;
use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::texture_files::{AssetTextureFiles, TerrainTextureFiles};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};
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

    /// Get terrain point count for mesh generation and memory allocation.
    pub fn terrain_point_count(&self) -> usize {
        self.terrain.point_count
    }

    /// Calculate terrain center point for camera positioning and navigation.
    pub fn terrain_center(&self) -> Vec3 {
        let bounds = &self.terrain.bounds;
        Vec3::new(
            ((bounds.max_x + bounds.min_x) * 0.5) as f32,
            ((bounds.max_y + bounds.min_y) * 0.5) as f32,
            ((bounds.max_z + bounds.min_z) * 0.5) as f32,
        )
    }

    /// Calculate terrain size dimensions for scene scaling and LOD calculations.
    pub fn terrain_size(&self) -> Vec3 {
        let bounds = &self.terrain.bounds;
        Vec3::new(
            (bounds.max_x - bounds.min_x) as f32,
            (bounds.max_y - bounds.min_y) as f32,
            (bounds.max_z - bounds.min_z) as f32,
        )
    }

    /// Get ground height for camera positioning and collision detection.
    pub fn ground_height(&self) -> f32 {
        self.terrain.bounds.min_y as f32
    }

    /// Find specific asset by name for runtime placement and interaction queries.
    pub fn get_asset_by_name(&self, name: &str) -> Option<&AssetDefinition> {
        self.asset_atlas
            .as_ref()?
            .assets
            .iter()
            .find(|asset| asset.name == name)
    }

    /// Calculate total asset points for memory allocation and performance tuning.
    pub fn total_asset_points(&self) -> usize {
        self.asset_atlas
            .as_ref()
            .map(|atlas| atlas.assets.iter().map(|a| a.point_count).sum())
            .unwrap_or(0)
    }

    /// Check if scene contains assets for conditional rendering pipeline setup.
    pub fn has_assets(&self) -> bool {
        self.asset_atlas.is_some()
    }

    /// Get terrain texture file paths for dynamic texture loading.
    pub fn terrain_texture_paths(&self) -> &TerrainTextureFiles {
        &self.terrain.texture_files
    }

    /// Get asset texture file paths when assets are present.
    pub fn asset_texture_paths(&self) -> Option<&AssetTextureFiles> {
        self.asset_atlas.as_ref().map(|atlas| &atlas.texture_files)
    }

    /// Extract legacy PointCloudBounds for compatibility with existing systems.
    /// Use sparingly - prefer direct manifest access for new code.
    pub fn to_point_cloud_bounds(&self) -> PointCloudBounds {
        PointCloudBounds {
            bounds: self.terrain.bounds.clone(),
            total_points: self.terrain.point_count,
            loaded_points: self.terrain.point_count,
            texture_size: 2048,  // Standard unified texture resolution.
            sampling_ratio: 1.0, // Full resolution for new manifests.
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
