use crate::laz::create_reader;
use constants::coordinate_system::transform_coordinates;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::{ParallelIterator, ParallelSlice};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Point cloud coordinate bounds tracking and normalisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointCloudBounds {
    /// Origin offset from LAZ file (to be subtracted from all points to maintain precision)
    pub origin: (f64, f64, f64),
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl PointCloudBounds {
    /// Create new bounds initialised to infinity values
    pub fn new(origin: (f64, f64, f64)) -> Self {
        Self {
            origin,
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
        let rel_x = x - self.origin.0;
        let rel_y = y - self.origin.1;
        let rel_z = z - self.origin.2;

        self.min_x = self.min_x.min(rel_x);
        self.max_x = self.max_x.max(rel_x);
        self.min_y = self.min_y.min(rel_y);
        self.max_y = self.max_y.max(rel_y);
        self.min_z = self.min_z.min(rel_z);
        self.max_z = self.max_z.max(rel_z);
    }

    /// Normalise X coordinate to 0-1 range
    pub fn normalize_x(&self, x: f64) -> f32 {
        let rel_x = x - self.origin.0;
        ((rel_x - self.min_x) / (self.max_x - self.min_x)) as f32
    }

    /// Normalise Y coordinate to 0-1 range
    pub fn normalize_y(&self, y: f64) -> f32 {
        let rel_y = y - self.origin.1;
        ((rel_y - self.min_y) / (self.max_y - self.min_y)) as f32
    }

    /// Normalise Z coordinate to 0-1 range
    pub fn normalize_z(&self, z: f64) -> f32 {
        let rel_z = z - self.origin.2;
        ((rel_z - self.min_z) / (self.max_z - self.min_z)) as f32
    }
}

/// Calculate coordinate bounds from all points with parallel processing.
/// Uses chunked parallel computation for efficient large dataset handling.
pub fn calculate_bounds(
    file_path: &Path,
    use_header_transform: bool,
) -> Result<PointCloudBounds, Box<dyn std::error::Error>> {
    let mut reader = create_reader(file_path)?;
    let total_points = reader.header().number_of_points() as usize;

    // Extract LAZ offsets - these will be our origin (placable assets are assumed to have NO header transform)
    let (x_offset, y_offset, z_offset) = if use_header_transform {
        let transforms = reader.header().transforms();
        (
            transforms.x.offset,
            transforms.y.offset,
            transforms.z.offset,
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    let origin = transform_coordinates(x_offset, y_offset, z_offset);

    // Load points with progress tracking.
    let pb = ProgressBar::new(total_points as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}] {pos}/{len} points ({percent}%) {msg}")
            .unwrap()
            .progress_chars("▉▊▋▌▍▎▏ "),
    );
    pb.set_message("Loading points");

    let mut all_points = Vec::with_capacity(total_points);
    for (idx, point_result) in reader.points().enumerate() {
        all_points.push(point_result?);

        if idx % 50_000 == 0 {
            pb.set_position(idx as u64);
        }
    }
    pb.finish_with_message("Points loaded");

    // Process bounds calculation in parallel chunks for efficiency.
    let pb = ProgressBar::new(all_points.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.green/blue}] {pos}/{len} chunks ({percent}%) {msg}")
            .unwrap()
            .progress_chars("▉▊▋▌▍▎▏ "),
    );
    pb.set_message("Calculating bounds");

    let bounds = all_points
        .par_chunks(25_000)
        .map(|chunk| {
            let mut local_bounds = PointCloudBounds::new(origin);
            for point in chunk {
                let (x, y, z) = transform_coordinates(point.x, point.y, point.z);
                local_bounds.update(x, y, z);
            }

            pb.inc(chunk.len() as u64);
            local_bounds
        })
        .reduce_with(|mut a, b| {
            a.min_x = a.min_x.min(b.min_x);
            a.max_x = a.max_x.max(b.max_x);
            a.min_y = a.min_y.min(b.min_y);
            a.max_y = a.max_y.max(b.max_y);
            a.min_z = a.min_z.min(b.min_z);
            a.max_z = a.max_z.max(b.max_z);
            a
        })
        .unwrap_or_else(|| PointCloudBounds::new(origin));

    pb.finish_with_message("Bounds calculated");
    Ok(bounds)
}
