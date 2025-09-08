use bevy::prelude::*;
use serde::{Deserialize, Serialize};
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

fn default_utilisation() -> f64 {
    0.0
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
