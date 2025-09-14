use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

// Crate tools modules
use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::scene_manifest::SceneManifest;

pub fn create_point_cloud_assets(manifest: Option<Handle<SceneManifest>>) -> PointCloudAssets {
    PointCloudAssets {
        position_texture: Handle::default(),
        colour_class_texture: Handle::default(),
        spatial_index_texture: Handle::default(),
        result_texture: Handle::default(),
        depth_texture: Handle::default(),
        heightmap_texture: Handle::default(),
        asset_position_texture: Handle::default(),
        asset_colour_class_texture: Handle::default(),
        manifest,
        is_loaded: false,
    }
}

/// Point cloud assets using unified texture format with terrain and asset support.
/// Manages all textures and scene metadata for the rendering pipeline.
#[derive(Resource, FromWorld, ExtractResource, Clone)]
pub struct PointCloudAssets {
    // Terrain textures - always present for base point cloud rendering.
    pub position_texture: Handle<Image>, // RGBA32F: XYZ + connectivity class id.
    pub colour_class_texture: Handle<Image>, // RGBA32F: RGB + classification.
    pub spatial_index_texture: Handle<Image>, // RG32Uint: spatial data.
    pub heightmap_texture: Handle<Image>, // R32F: elevation.

    // Asset atlas textures
    pub asset_position_texture: Handle<Image>,
    pub asset_colour_class_texture: Handle<Image>,

    // Compute pipeline textures for classification and EDL processing.
    pub depth_texture: Handle<Image>,  // R32F: R = Depth.
    pub result_texture: Handle<Image>, // RGBA32F: RenderMode = RGB + A = Depth.

    // Scene manifest contains all metadata including terrain bounds and assets.
    pub manifest: Option<Handle<SceneManifest>>,
    pub is_loaded: bool,
}

impl PointCloudAssets {
    /// Extract bounds from manifest for systems expecting legacy PointCloudBounds.
    /// Returns None if manifest is not loaded yet. Prefer direct manifest access.
    pub fn get_bounds(&self, manifests: &Assets<SceneManifest>) -> Option<PointCloudBounds> {
        let manifest_handle = self.manifest.as_ref()?;
        let manifest = manifests.get(manifest_handle)?;
        Some(manifest.to_point_cloud_bounds())
    }

    /// Check if manifest has been loaded and assets are ready for rendering.
    pub fn is_manifest_loaded(&self, manifests: &Assets<SceneManifest>) -> bool {
        if let Some(handle) = &self.manifest {
            manifests.get(handle).is_some()
        } else {
            false
        }
    }

    /// Get terrain point count from manifest for mesh generation.
    /// Returns 0 if manifest not loaded yet.
    pub fn terrain_point_count(&self, manifests: &Assets<SceneManifest>) -> usize {
        self.manifest
            .as_ref()
            .and_then(|h| manifests.get(h))
            .map(|m| m.terrain_point_count())
            .unwrap_or(0)
    }
}
