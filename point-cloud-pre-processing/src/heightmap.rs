// /// Fast parallel heightmap generation with smooth blending
// use crate::constants::{HEIGHTMAP_BLEND_RADIUS, TEXTURE_SIZE};
// use crate::dds_writer::write_r32f_texture;
// use indicatif::{ProgressBar, ProgressStyle};
// use rayon::prelude::*;
// use std::sync::atomic::{AtomicUsize, Ordering};

// pub struct HeightmapGenerator {
//     output_stem: String,
// }

// impl HeightmapGenerator {
//     pub fn new(output_stem: &str) -> Self {
//         Self {
//             output_stem: output_stem.to_string(),
//         }
//     }

//     /// Generate unified resolution road heightmap with parallel processing
//     pub fn generate_unified(
//         &self,
//         road_points: &[(f32, f32, f32)], // (x, z, y) normalised coordinates
//         average_elevation: f32,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         println!(
//             "Creating {}x{} heightmap from {} road points (parallel)",
//             TEXTURE_SIZE,
//             TEXTURE_SIZE,
//             road_points.len()
//         );

//         // Convert to grid coordinates once
//         let grid_points: Vec<(i32, i32, f32)> = road_points
//             .iter()
//             .map(|&(x, z, y)| {
//                 let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
//                 let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;
//                 (grid_x, grid_z, y)
//             })
//             .collect();

//         // Use distance field approach for smooth blending
//         let height_data =
//             self.generate_distance_field_heightmap(&grid_points, average_elevation)?;

//         let heightmap_path = format!(
//             "{}_heightmap_{}x{}.dds",
//             self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
//         );
//         write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;

//         println!("Saved {} (R32F heightmap)", heightmap_path);
//         Ok(())
//     }

//     /// Fast distance field based heightmap generation
//     fn generate_distance_field_heightmap(
//         &self,
//         grid_points: &[(i32, i32, f32)],
//         average_elevation: f32,
//     ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
//         let total_pixels = TEXTURE_SIZE * TEXTURE_SIZE;

//         // Progress tracking
//         let progress = AtomicUsize::new(0);
//         let pb = ProgressBar::new(total_pixels as u64);
//         pb.set_style(
//             ProgressStyle::default_bar()
//                 .template("[{bar:40.cyan/blue}] {pos}/{len} pixels ({percent}%) {msg}")
//                 .unwrap()
//                 .progress_chars("█▉▊▋▌▍▎▏ "),
//         );
//         pb.set_message("Generating heightmap");

//         // Process in parallel chunks
//         let chunk_size = 1024; // Process 1024 pixels per chunk
//         let chunks: Vec<_> = (0..total_pixels)
//             .collect::<Vec<_>>()
//             .chunks(chunk_size)
//             .map(|chunk| chunk.to_vec())
//             .collect();

//         let height_data: Vec<f32> = chunks
//             .par_iter()
//             .flat_map(|chunk| {
//                 let mut local_heights = Vec::with_capacity(chunk.len());

//                 for &pixel_idx in chunk {
//                     let x = (pixel_idx % TEXTURE_SIZE) as i32;
//                     let z = (pixel_idx / TEXTURE_SIZE) as i32;

//                     let height = self.calculate_pixel_height(x, z, grid_points, average_elevation);
//                     local_heights.push(height);
//                 }

//                 // Update progress
//                 let completed = progress.fetch_add(chunk.len(), Ordering::Relaxed) + chunk.len();
//                 pb.set_position(completed as u64);

//                 local_heights
//             })
//             .collect();

//         pb.finish_with_message("Heightmap complete");
//         Ok(height_data)
//     }

//     /// Calculate height for a single pixel using weighted distance
//     fn calculate_pixel_height(
//         &self,
//         x: i32,
//         z: i32,
//         grid_points: &[(i32, i32, f32)],
//         average_elevation: f32,
//     ) -> f32 {
//         let max_distance = HEIGHTMAP_BLEND_RADIUS;
//         let mut total_weight = 0.0f32;
//         let mut weighted_height = 0.0f32;

//         // Find nearby points within blend radius
//         for &(px, pz, height) in grid_points {
//             let dx = (x - px) as f32;
//             let dz = (z - pz) as f32;
//             let distance = (dx * dx + dz * dz).sqrt();

//             if distance <= max_distance {
//                 // Smooth falloff using cosine interpolation
//                 let normalized_dist = distance / max_distance;
//                 let weight = if distance < 0.1 {
//                     1.0 // Very close points get full weight
//                 } else {
//                     (1.0 + (normalized_dist * std::f32::consts::PI).cos()) * 0.5
//                 };

//                 total_weight += weight;
//                 weighted_height += height * weight;
//             }
//         }

//         // Blend with average elevation
//         if total_weight > 0.0 {
//             let road_height = weighted_height / total_weight;
//             let blend_factor = (total_weight / 2.0).min(1.0); // Smooth transition
//             average_elevation * (1.0 - blend_factor) + road_height * blend_factor
//         } else {
//             average_elevation
//         }
//     }

//     /// Alternative: Super fast approximate heightmap using spatial hashing (recommended for large datasets)
//     pub fn generate_fast_approximate(
//         &self,
//         road_points: &[(f32, f32, f32)],
//         average_elevation: f32,
//     ) -> Result<(), Box<dyn std::error::Error>> {
//         println!("Generating fast approximate heightmap...");

//         let pb = ProgressBar::new(TEXTURE_SIZE as u64);
//         pb.set_style(
//             ProgressStyle::default_bar()
//                 .template("[{bar:40.green/blue}] {pos}/{len} rows ({percent}%) {msg}")
//                 .unwrap()
//                 .progress_chars("█▉▊▋▌▍▎▏ "),
//         );
//         pb.set_message("Processing rows");

//         // Spatial hash for O(1) lookups
//         let cell_size = HEIGHTMAP_BLEND_RADIUS as usize / 2;
//         let hash_size = (TEXTURE_SIZE / cell_size) + 1;
//         let mut spatial_hash: Vec<Vec<(i32, i32, f32)>> = vec![Vec::new(); hash_size * hash_size];

//         // Populate spatial hash
//         for &(x, z, y) in road_points {
//             let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
//             let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;

//             let hash_x = (grid_x as usize / cell_size).min(hash_size - 1);
//             let hash_z = (grid_z as usize / cell_size).min(hash_size - 1);
//             let hash_idx = hash_z * hash_size + hash_x;

//             spatial_hash[hash_idx].push((grid_x, grid_z, y));
//         }

//         // Process rows in parallel
//         let height_data: Vec<f32> = (0..TEXTURE_SIZE)
//             .into_par_iter()
//             .flat_map(|z| {
//                 let mut row_data = Vec::with_capacity(TEXTURE_SIZE);

//                 for x in 0..TEXTURE_SIZE {
//                     let height = self.calculate_pixel_height_hashed(
//                         x as i32,
//                         z as i32,
//                         &spatial_hash,
//                         hash_size,
//                         cell_size,
//                         average_elevation,
//                     );
//                     row_data.push(height);
//                 }

//                 pb.inc(1);
//                 row_data
//             })
//             .collect();

//         pb.finish_with_message("Fast heightmap complete");

//         let heightmap_path = format!(
//             "{}_heightmap_{}x{}.dds",
//             self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
//         );
//         write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;
//         println!("Saved {} (R32F heightmap)", heightmap_path);

//         Ok(())
//     }

//     fn calculate_pixel_height_hashed(
//         &self,
//         x: i32,
//         z: i32,
//         spatial_hash: &[Vec<(i32, i32, f32)>],
//         hash_size: usize,
//         cell_size: usize,
//         average_elevation: f32,
//     ) -> f32 {
//         let max_distance = HEIGHTMAP_BLEND_RADIUS;
//         let mut total_weight = 0.0f32;
//         let mut weighted_height = 0.0f32;

//         // Check surrounding hash cells
//         let center_hash_x = (x as usize / cell_size).min(hash_size - 1);
//         let center_hash_z = (z as usize / cell_size).min(hash_size - 1);

//         for dz in -1..=1i32 {
//             for dx in -1..=1i32 {
//                 let hash_x = (center_hash_x as i32 + dx).max(0).min(hash_size as i32 - 1) as usize;
//                 let hash_z = (center_hash_z as i32 + dz).max(0).min(hash_size as i32 - 1) as usize;
//                 let hash_idx = hash_z * hash_size + hash_x;

//                 for &(px, pz, height) in &spatial_hash[hash_idx] {
//                     let dist_x = (x - px) as f32;
//                     let dist_z = (z - pz) as f32;
//                     let distance = (dist_x * dist_x + dist_z * dist_z).sqrt();

//                     if distance <= max_distance {
//                         let weight =
//                             (-distance * distance / (max_distance * max_distance * 0.5)).exp();
//                         total_weight += weight;
//                         weighted_height += height * weight;
//                     }
//                 }
//             }
//         }

//         if total_weight > 0.0 {
//             let road_height = weighted_height / total_weight;
//             let blend_factor = (total_weight / 2.0).min(1.0);
//             average_elevation * (1.0 - blend_factor) + road_height * blend_factor
//         } else {
//             average_elevation
//         }
//     }
// }

/// Fast parallel heightmap generation with smooth blending
use crate::constants::TEXTURE_SIZE;
use crate::dds_writer::write_r32f_texture;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::VecDeque;

pub struct HeightmapGenerator {
    output_stem: String,
}

impl HeightmapGenerator {
    pub fn new(output_stem: &str) -> Self {
        Self {
            output_stem: output_stem.to_string(),
        }
    }

    /// Generate heightmap using flood-fill from pre-filtered road points
    pub fn generate_flood_fill_from_road_points(
        &self,
        road_points: &[(f32, f32, f32)], // (norm_x, norm_z, norm_y)
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Generating flood-fill heightmap from {} road points...",
            road_points.len()
        );

        // Calculate median elevation from road points
        let median_elevation = if road_points.is_empty() {
            0.5
        } else {
            let mut elevations: Vec<f32> = road_points.iter().map(|(_, _, y)| *y).collect();
            elevations.sort_by(|a, b| a.partial_cmp(b).unwrap());
            elevations[elevations.len() / 2]
        };

        println!("Using median road elevation: {:.3}", median_elevation);

        // Initialize heightmap and validity mask
        let mut heightmap = vec![median_elevation; TEXTURE_SIZE * TEXTURE_SIZE];
        let mut valid_mask = vec![false; TEXTURE_SIZE * TEXTURE_SIZE];

        // Place road points in grid
        for &(norm_x, norm_z, norm_y) in road_points {
            let grid_x = ((norm_x * (TEXTURE_SIZE - 1) as f32) as usize).min(TEXTURE_SIZE - 1);
            let grid_z = ((norm_z * (TEXTURE_SIZE - 1) as f32) as usize).min(TEXTURE_SIZE - 1);
            let idx = grid_z * TEXTURE_SIZE + grid_x;

            heightmap[idx] = norm_y;
            valid_mask[idx] = true;
        }

        // Flood fill gaps
        self.flood_fill_gaps(&mut heightmap, &valid_mask, median_elevation)?;

        // Apply smoothing
        let smoothed_heightmap = self.apply_gaussian_blur(&heightmap, 3.0)?;

        // Save heightmap
        let heightmap_path = format!(
            "{}_heightmap_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &smoothed_heightmap)?;
        println!("Saved {} (R32F heightmap)", heightmap_path);

        Ok(())
    }

    /// Flood fill to propagate valid elevations to nearby empty cells
    fn flood_fill_gaps(
        &self,
        heightmap: &mut [f32],
        valid_mask: &[bool],
        default_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pb = ProgressBar::new(TEXTURE_SIZE as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.yellow/blue}] {pos}/{len} flood-fill ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Flood filling gaps");

        let mut filled_mask = valid_mask.to_vec();
        let mut queue = VecDeque::new();

        // Initialize queue with all valid points
        for z in 0..TEXTURE_SIZE {
            for x in 0..TEXTURE_SIZE {
                let idx = z * TEXTURE_SIZE + x;
                if valid_mask[idx] {
                    queue.push_back((x, z));
                }
            }
        }

        // Propagate in waves
        while !queue.is_empty() {
            let (x, z) = queue.pop_front().unwrap();
            let current_idx = z * TEXTURE_SIZE + x;
            let current_height = heightmap[current_idx];

            // Check 8-connected neighbors
            for dz in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dx == 0 && dz == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let nz = z as i32 + dz;

                    if nx >= 0 && nx < TEXTURE_SIZE as i32 && nz >= 0 && nz < TEXTURE_SIZE as i32 {
                        let neighbor_idx = (nz as usize) * TEXTURE_SIZE + (nx as usize);

                        if !filled_mask[neighbor_idx] {
                            let distance = ((dx * dx + dz * dz) as f32).sqrt();
                            let weight = (-distance * 0.5).exp();

                            heightmap[neighbor_idx] =
                                current_height * weight + default_elevation * (1.0 - weight);
                            filled_mask[neighbor_idx] = true;
                            queue.push_back((nx as usize, nz as usize));
                        }
                    }
                }
            }

            if z % 32 == 0 {
                pb.set_position(z as u64);
            }
        }

        pb.finish_with_message("Flood fill complete");
        Ok(())
    }

    /// Apply Gaussian blur for smooth transitions
    fn apply_gaussian_blur(
        &self,
        heightmap: &[f32],
        sigma: f32,
    ) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let pb = ProgressBar::new(TEXTURE_SIZE as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} blur ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Applying smooth blur");

        // Generate Gaussian kernel
        let kernel_size = (sigma * 3.0).ceil() as i32;
        let mut kernel = Vec::new();
        let mut kernel_sum = 0.0f32;

        for i in -kernel_size..=kernel_size {
            let value = (-((i * i) as f32) / (2.0 * sigma * sigma)).exp();
            kernel.push(value);
            kernel_sum += value;
        }

        for k in &mut kernel {
            *k /= kernel_sum;
        }

        // Two-pass separable Gaussian blur
        let mut temp_buffer = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE];

        // Horizontal pass
        temp_buffer
            .par_chunks_mut(TEXTURE_SIZE)
            .enumerate()
            .for_each(|(z, row)| {
                for x in 0..TEXTURE_SIZE {
                    let mut sum = 0.0f32;

                    for (ki, &k_val) in kernel.iter().enumerate() {
                        let offset = ki as i32 - kernel_size;
                        let sample_x =
                            (x as i32 + offset).clamp(0, TEXTURE_SIZE as i32 - 1) as usize;
                        sum += heightmap[z * TEXTURE_SIZE + sample_x] * k_val;
                    }

                    row[x] = sum;
                }
            });

        // Vertical pass
        let mut result = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE];

        result
            .par_chunks_mut(TEXTURE_SIZE)
            .enumerate()
            .for_each(|(z, row)| {
                for x in 0..TEXTURE_SIZE {
                    let mut sum = 0.0f32;

                    for (ki, &k_val) in kernel.iter().enumerate() {
                        let offset = ki as i32 - kernel_size;
                        let sample_z =
                            (z as i32 + offset).clamp(0, TEXTURE_SIZE as i32 - 1) as usize;
                        sum += temp_buffer[sample_z * TEXTURE_SIZE + x] * k_val;
                    }

                    row[x] = sum;
                }

                if z % 64 == 0 {
                    pb.inc(64);
                }
            });

        pb.finish_with_message("Blur complete");
        Ok(result)
    }
}
