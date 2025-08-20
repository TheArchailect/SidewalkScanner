// /// Road surface heightmap generation with smooth blending
// use crate::constants::{HEIGHTMAP_BLEND_RADIUS, TEXTURE_SIZE};
// use crate::dds_writer::write_r32f_texture;

// pub struct HeightmapGenerator {
//     output_stem: String,
// }

// impl HeightmapGenerator {
//     /// Create new heightmap generator
//     pub fn new(output_stem: &str) -> Self {
//         Self {
//             output_stem: output_stem.to_string(),
//         }
//     }

//     /// Generate unified resolution road heightmap
//     pub fn generate_unified(
//         &self,
//         road_points: &[(f32, f32, f32)], // (x, z, y) normalised coordinates
//         average_elevation: f32,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         println!(
//             "Creating {}x{} road heightmap from {} road points",
//             TEXTURE_SIZE,
//             TEXTURE_SIZE,
//             road_points.len()
//         );

//         // Initialise with average elevation
//         let mut height_data = vec![average_elevation; TEXTURE_SIZE * TEXTURE_SIZE];
//         let mut influence_map = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE];
//         let mut road_height_map = vec![average_elevation; TEXTURE_SIZE * TEXTURE_SIZE];

//         // Apply road heights with influence weighting
//         for &(x, z, y) in road_points {
//             let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
//             let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;

//             self.apply_height_with_blend(
//                 &mut road_height_map,
//                 &mut influence_map,
//                 grid_x,
//                 grid_z,
//                 y,
//             );
//         }

//         // Blend road heights with base elevation
//         for i in 0..height_data.len() {
//             let influence = influence_map[i].min(1.0);
//             height_data[i] = average_elevation * (1.0 - influence) + road_height_map[i] * influence;
//         }

//         let heightmap_path = format!(
//             "{}_heightmap_{}x{}.dds",
//             self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
//         );
//         write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;

//         println!("Saved {} (R32F heightmap)", heightmap_path);
//         Ok(())
//     }

//     /// Apply height value with gaussian blend in radius
//     fn apply_height_with_blend(
//         &self,
//         road_height_map: &mut [f32],
//         influence_map: &mut [f32],
//         center_x: i32,
//         center_z: i32,
//         height: f32,
//     ) {
//         let radius = HEIGHTMAP_BLEND_RADIUS as i32;

//         for dz in -radius..=radius {
//             for dx in -radius..=radius {
//                 let px = center_x + dx;
//                 let pz = center_z + dz;

//                 if self.is_valid_coordinate(px, pz) {
//                     let distance = ((dx * dx + dz * dz) as f32).sqrt();
//                     if distance <= HEIGHTMAP_BLEND_RADIUS {
//                         let pixel_index = (pz as usize) * TEXTURE_SIZE + (px as usize);
//                         let influence = self.calculate_influence(distance);

//                         self.apply_weighted_height(
//                             road_height_map,
//                             influence_map,
//                             pixel_index,
//                             height,
//                             influence,
//                         );
//                     }
//                 }
//             }
//         }
//     }

//     /// Check if coordinates are within texture bounds
//     fn is_valid_coordinate(&self, x: i32, z: i32) -> bool {
//         x >= 0 && x < TEXTURE_SIZE as i32 && z >= 0 && z < TEXTURE_SIZE as i32
//     }

//     /// Calculate gaussian influence based on distance
//     fn calculate_influence(&self, distance: f32) -> f32 {
//         (-distance * distance / (HEIGHTMAP_BLEND_RADIUS * HEIGHTMAP_BLEND_RADIUS * 0.5)).exp()
//     }

//     /// Apply weighted height value using influence
//     fn apply_weighted_height(
//         &self,
//         road_height_map: &mut [f32],
//         influence_map: &mut [f32],
//         pixel_index: usize,
//         new_height: f32,
//         influence: f32,
//     ) {
//         let current_influence = influence_map[pixel_index];
//         let new_total_influence = current_influence + influence;

//         if new_total_influence > 0.0 {
//             road_height_map[pixel_index] = (road_height_map[pixel_index] * current_influence
//                 + new_height * influence)
//                 / new_total_influence;
//             influence_map[pixel_index] = new_total_influence;
//         }
//     }
// }

/// Fast parallel heightmap generation with smooth blending
use crate::constants::{HEIGHTMAP_BLEND_RADIUS, TEXTURE_SIZE};
use crate::dds_writer::write_r32f_texture;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct HeightmapGenerator {
    output_stem: String,
}

impl HeightmapGenerator {
    pub fn new(output_stem: &str) -> Self {
        Self {
            output_stem: output_stem.to_string(),
        }
    }

    /// Generate unified resolution road heightmap with parallel processing
    pub fn generate_unified(
        &self,
        road_points: &[(f32, f32, f32)], // (x, z, y) normalised coordinates
        average_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Creating {}x{} heightmap from {} road points (parallel)",
            TEXTURE_SIZE,
            TEXTURE_SIZE,
            road_points.len()
        );

        // Convert to grid coordinates once
        let grid_points: Vec<(i32, i32, f32)> = road_points
            .iter()
            .map(|&(x, z, y)| {
                let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
                let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;
                (grid_x, grid_z, y)
            })
            .collect();

        // Use distance field approach for smooth blending
        let height_data =
            self.generate_distance_field_heightmap(&grid_points, average_elevation)?;

        let heightmap_path = format!(
            "{}_heightmap_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;

        println!("Saved {} (R32F heightmap)", heightmap_path);
        Ok(())
    }

    /// Fast distance field based heightmap generation
    fn generate_distance_field_heightmap(
        &self,
        grid_points: &[(i32, i32, f32)],
        average_elevation: f32,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let total_pixels = TEXTURE_SIZE * TEXTURE_SIZE;

        // Progress tracking
        let progress = AtomicUsize::new(0);
        let pb = ProgressBar::new(total_pixels as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} pixels ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Generating heightmap");

        // Process in parallel chunks
        let chunk_size = 1024; // Process 1024 pixels per chunk
        let chunks: Vec<_> = (0..total_pixels)
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect();

        let height_data: Vec<f32> = chunks
            .par_iter()
            .flat_map(|chunk| {
                let mut local_heights = Vec::with_capacity(chunk.len());

                for &pixel_idx in chunk {
                    let x = (pixel_idx % TEXTURE_SIZE) as i32;
                    let z = (pixel_idx / TEXTURE_SIZE) as i32;

                    let height = self.calculate_pixel_height(x, z, grid_points, average_elevation);
                    local_heights.push(height);
                }

                // Update progress
                let completed = progress.fetch_add(chunk.len(), Ordering::Relaxed) + chunk.len();
                pb.set_position(completed as u64);

                local_heights
            })
            .collect();

        pb.finish_with_message("Heightmap complete");
        Ok(height_data)
    }

    /// Calculate height for a single pixel using weighted distance
    fn calculate_pixel_height(
        &self,
        x: i32,
        z: i32,
        grid_points: &[(i32, i32, f32)],
        average_elevation: f32,
    ) -> f32 {
        let max_distance = HEIGHTMAP_BLEND_RADIUS;
        let mut total_weight = 0.0f32;
        let mut weighted_height = 0.0f32;

        // Find nearby points within blend radius
        for &(px, pz, height) in grid_points {
            let dx = (x - px) as f32;
            let dz = (z - pz) as f32;
            let distance = (dx * dx + dz * dz).sqrt();

            if distance <= max_distance {
                // Smooth falloff using cosine interpolation
                let normalized_dist = distance / max_distance;
                let weight = if distance < 0.1 {
                    1.0 // Very close points get full weight
                } else {
                    (1.0 + (normalized_dist * std::f32::consts::PI).cos()) * 0.5
                };

                total_weight += weight;
                weighted_height += height * weight;
            }
        }

        // Blend with average elevation
        if total_weight > 0.0 {
            let road_height = weighted_height / total_weight;
            let blend_factor = (total_weight / 2.0).min(1.0); // Smooth transition
            average_elevation * (1.0 - blend_factor) + road_height * blend_factor
        } else {
            average_elevation
        }
    }

    /// Alternative: Super fast approximate heightmap using spatial hashing (recommended for large datasets)
    pub fn generate_fast_approximate(
        &self,
        road_points: &[(f32, f32, f32)],
        average_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating fast approximate heightmap...");

        let pb = ProgressBar::new(TEXTURE_SIZE as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} rows ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Processing rows");

        // Spatial hash for O(1) lookups
        let cell_size = HEIGHTMAP_BLEND_RADIUS as usize / 2;
        let hash_size = (TEXTURE_SIZE / cell_size) + 1;
        let mut spatial_hash: Vec<Vec<(i32, i32, f32)>> = vec![Vec::new(); hash_size * hash_size];

        // Populate spatial hash
        for &(x, z, y) in road_points {
            let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
            let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;

            let hash_x = (grid_x as usize / cell_size).min(hash_size - 1);
            let hash_z = (grid_z as usize / cell_size).min(hash_size - 1);
            let hash_idx = hash_z * hash_size + hash_x;

            spatial_hash[hash_idx].push((grid_x, grid_z, y));
        }

        // Process rows in parallel
        let height_data: Vec<f32> = (0..TEXTURE_SIZE)
            .into_par_iter()
            .flat_map(|z| {
                let mut row_data = Vec::with_capacity(TEXTURE_SIZE);

                for x in 0..TEXTURE_SIZE {
                    let height = self.calculate_pixel_height_hashed(
                        x as i32,
                        z as i32,
                        &spatial_hash,
                        hash_size,
                        cell_size,
                        average_elevation,
                    );
                    row_data.push(height);
                }

                pb.inc(1);
                row_data
            })
            .collect();

        pb.finish_with_message("Fast heightmap complete");

        let heightmap_path = format!(
            "{}_heightmap_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;
        println!("Saved {} (R32F heightmap)", heightmap_path);

        Ok(())
    }

    fn calculate_pixel_height_hashed(
        &self,
        x: i32,
        z: i32,
        spatial_hash: &[Vec<(i32, i32, f32)>],
        hash_size: usize,
        cell_size: usize,
        average_elevation: f32,
    ) -> f32 {
        let max_distance = HEIGHTMAP_BLEND_RADIUS;
        let mut total_weight = 0.0f32;
        let mut weighted_height = 0.0f32;

        // Check surrounding hash cells
        let center_hash_x = (x as usize / cell_size).min(hash_size - 1);
        let center_hash_z = (z as usize / cell_size).min(hash_size - 1);

        for dz in -1..=1i32 {
            for dx in -1..=1i32 {
                let hash_x = (center_hash_x as i32 + dx).max(0).min(hash_size as i32 - 1) as usize;
                let hash_z = (center_hash_z as i32 + dz).max(0).min(hash_size as i32 - 1) as usize;
                let hash_idx = hash_z * hash_size + hash_x;

                for &(px, pz, height) in &spatial_hash[hash_idx] {
                    let dist_x = (x - px) as f32;
                    let dist_z = (z - pz) as f32;
                    let distance = (dist_x * dist_x + dist_z * dist_z).sqrt();

                    if distance <= max_distance {
                        let weight =
                            (-distance * distance / (max_distance * max_distance * 0.5)).exp();
                        total_weight += weight;
                        weighted_height += height * weight;
                    }
                }
            }
        }

        if total_weight > 0.0 {
            let road_height = weighted_height / total_weight;
            let blend_factor = (total_weight / 2.0).min(1.0);
            average_elevation * (1.0 - blend_factor) + road_height * blend_factor
        } else {
            average_elevation
        }
    }
}
