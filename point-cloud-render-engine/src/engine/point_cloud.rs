use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct PointCloud;

/// Point cloud assets using unified texture format with terrain and asset support.
/// Manages all textures and scene metadata for the rendering pipeline.
#[derive(Resource, FromWorld, ExtractResource, Clone)]
pub struct PointCloudAssets {
    // Terrain textures - always present for base point cloud rendering.
    pub position_texture: Handle<Image>, // RGBA32F: XYZ + connectivity class id.
    pub colour_class_texture: Handle<Image>, // RGBA32F: RGB + classification.
    pub spatial_index_texture: Handle<Image>, // RG32Uint: spatial data.
    pub heightmap_texture: Handle<Image>, // R32F: elevation.

    // Asset atlas textures - populated when manifest contains asset_atlas.
    pub asset_position_texture: Option<Handle<Image>>,
    pub asset_colour_class_texture: Option<Handle<Image>>,

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

/// Terrain point cloud data with texture file references and metadata.
/// Contains the core point cloud information that was previously in bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainData {
    pub texture_files: TerrainTextureFiles,
    pub bounds: BoundsData,
    pub point_count: usize,
    pub has_colour: bool,
}

/// DDS texture file paths for terrain point cloud data.
/// Replaces hardcoded texture path construction in loading systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainTextureFiles {
    pub position: String,
    pub colour_class: String,
    pub spatial_index: String,
    pub heightmap: String,
}

/// Asset atlas containing texture references and individual asset definitions.
/// Enables placement of 3D objects within the point cloud scene.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAtlasData {
    pub texture_files: AssetTextureFiles,
    pub assets: Vec<AssetDefinition>,
    pub atlas_config: AtlasConfig,
}

/// DDS texture file paths for asset atlas data.
/// Asset atlas uses separate textures from terrain for independent resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTextureFiles {
    pub position: String,
    pub colour_class: String,
}

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

/// Legacy point cloud bounds for compatibility with existing rendering systems.
/// Contains metadata fields that existing systems expect for rendering setup.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Asset, TypePath)]
pub struct PointCloudBounds {
    pub bounds: BoundsData,
    pub total_points: usize,
    pub loaded_points: usize,
    pub texture_size: u32,
    #[serde(default)]
    pub sampling_ratio: f64,
    #[serde(default = "default_utilisation")]
    pub utilisation_percent: f64,
    #[serde(default)]
    pub has_colour: bool,
    #[serde(default)]
    pub colour_points: usize,
    #[serde(default)]
    pub road_points: usize,
}

impl PointCloudBounds {
    /// Calculate center point for camera positioning and scene navigation.
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            ((self.bounds.max_x + self.bounds.min_x) * 0.5) as f32,
            ((self.bounds.max_y + self.bounds.min_y) * 0.5) as f32,
            ((self.bounds.max_z + self.bounds.min_z) * 0.5) as f32,
        )
    }

    /// Calculate size dimensions for frustum culling and LOD calculations.
    pub fn size(&self) -> Vec3 {
        Vec3::new(
            (self.bounds.max_x - self.bounds.min_x) as f32,
            (self.bounds.max_y - self.bounds.min_y) as f32,
            (self.bounds.max_z - self.bounds.min_z) as f32,
        )
    }

    /// Get ground height for camera collision and terrain placement.
    pub fn ground_height(&self) -> f32 {
        self.bounds.min_y as f32
    }

    // Direct bounds accessors for performance-critical rendering queries.
    pub fn min_x(&self) -> f64 {
        self.bounds.min_x
    }
    pub fn max_x(&self) -> f64 {
        self.bounds.max_x
    }
    pub fn min_y(&self) -> f64 {
        self.bounds.min_y
    }
    pub fn max_y(&self) -> f64 {
        self.bounds.max_y
    }
    pub fn min_z(&self) -> f64 {
        self.bounds.min_z
    }
    pub fn max_z(&self) -> f64 {
        self.bounds.max_z
    }
}

/// 3D spatial bounds defining scene extents in world coordinates.
/// Used by both terrain and individual assets for positioning calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundsData {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

fn default_utilisation() -> f64 {
    0.0
}

/// Create point index mesh for GPU-side vertex expansion in custom pipeline.
/// Generates triangle-based geometry that expands to screen-aligned quads per point.
pub fn create_point_index_mesh(point_count: usize) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, // Triangle-based point rendering.
        RenderAssetUsages::RENDER_WORLD,
    );

    // Generate 6 vertices per point (2 triangles forming screen-aligned quad).
    // Vertex shader uses vertex index to determine point and triangle position.
    let vertex_count = point_count * 6;
    let indices: Vec<[f32; 3]> = (0..vertex_count).map(|i| [i as f32, 0.0, 0.0]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
    mesh
}
