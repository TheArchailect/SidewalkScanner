/// Point cloud coordinate bounds tracking and normalisation
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloudBounds {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl PointCloudBounds {
    /// Create new bounds initialised to infinity values
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            min_y: f64::INFINITY,
            max_y: f64::NEG_INFINITY,
            min_z: f64::INFINITY,
            max_z: f64::NEG_INFINITY,
        }
    }

    /// Update bounds with a new point
    pub fn update(&mut self, x: f64, y: f64, z: f64) {
        self.min_x = self.min_x.min(x);
        self.max_x = self.max_x.max(x);
        self.min_y = self.min_y.min(y);
        self.max_y = self.max_y.max(y);
        self.min_z = self.min_z.min(z);
        self.max_z = self.max_z.max(z);
    }

    /// Get world space dimensions - ADD THIS
    pub fn dimensions(&self) -> (f64, f64, f64) {
        (
            self.max_x - self.min_x,
            self.max_y - self.min_y,
            self.max_z - self.min_z,
        )
    }

    /// Normalise X coordinate to 0-1 range
    pub fn normalize_x(&self, x: f64) -> f32 {
        ((x - self.min_x) / (self.max_x - self.min_x)) as f32
    }

    /// Normalise Y coordinate to 0-1 range
    pub fn normalize_y(&self, y: f64) -> f32 {
        ((y - self.min_y) / (self.max_y - self.min_y)) as f32
    }

    /// Normalise Z coordinate to 0-1 range
    pub fn normalize_z(&self, z: f64) -> f32 {
        ((z - self.min_z) / (self.max_z - self.min_z)) as f32
    }
}
