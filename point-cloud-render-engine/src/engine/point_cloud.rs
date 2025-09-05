use bevy::render::extract_resource::ExtractResource;

use bevy::prelude::*;
use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};

#[derive(Component)]
pub struct PointCloud;

/// Point cloud assets using unified texture format
#[derive(Resource, FromWorld, ExtractResource, Clone)]
pub struct PointCloudAssets {
    pub position_texture: Handle<Image>, // RGBA32F: XYZ + connectivity class id
    pub colour_class_texture: Handle<Image>, // RGBA32F: RGB + classification (original)
    pub spatial_index_texture: Handle<Image>, // RG32Uint: spatial data
    pub heightmap_texture: Handle<Image>, // R32F: elevation

    // Compute Phase: 1
    pub depth_texture: Handle<Image>, // R32F: R = Depth
    // Compute Phase: 2
    pub result_texture: Handle<Image>, // RGBA32F: RenderMode = RGB (modified) + A = Depth

    // to do
    // pub selected_connectivity_classes: ,
    pub bounds: Option<PointCloudBounds>,
    pub is_loaded: bool,
}

use serde::{Deserialize, Serialize};

/// Point cloud bounds and metadata from unified texture pipeline
#[derive(Resource, Debug, Clone, Serialize, Deserialize, bevy::asset::Asset, TypePath)]
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

impl PointCloudBounds {
    /// Calculate centre point of bounds
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            ((self.bounds.max_x + self.bounds.min_x) * 0.5) as f32,
            ((self.bounds.max_y + self.bounds.min_y) * 0.5) as f32,
            ((self.bounds.max_z + self.bounds.min_z) * 0.5) as f32,
        )
    }

    /// Calculate size dimensions
    pub fn size(&self) -> Vec3 {
        Vec3::new(
            (self.bounds.max_x - self.bounds.min_x) as f32,
            (self.bounds.max_y - self.bounds.min_y) as f32,
            (self.bounds.max_z - self.bounds.min_z) as f32,
        )
    }

    /// Get ground height for camera positioning
    pub fn ground_height(&self) -> f32 {
        self.bounds.min_y as f32
    }

    // Convenience accessors for direct bounds access
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

pub fn create_point_index_mesh(point_count: usize) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, // Changed from PointList
        RenderAssetUsages::RENDER_WORLD,
    );
    let vertex_count = point_count * 6; // 6 vertices per point (2 triangles)
    let indices: Vec<[f32; 3]> = (0..vertex_count).map(|i| [i as f32, 0.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
    mesh
}
