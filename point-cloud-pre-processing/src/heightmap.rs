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

        // Initialise queue with all valid points
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
