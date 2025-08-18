/// Road surface heightmap generation with smooth blending
use crate::constants::{HEIGHTMAP_BLEND_RADIUS, TEXTURE_SIZE};
use crate::dds_writer::write_r32f_texture;

pub struct HeightmapGenerator {
    output_stem: String,
}

impl HeightmapGenerator {
    /// Create new heightmap generator
    pub fn new(output_stem: &str) -> Self {
        Self {
            output_stem: output_stem.to_string(),
        }
    }

    /// Generate unified resolution road heightmap
    pub fn generate_unified(
        &self,
        road_points: &[(f32, f32, f32)], // (x, z, y) normalised coordinates
        average_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Creating {}x{} road heightmap from {} road points",
            TEXTURE_SIZE,
            TEXTURE_SIZE,
            road_points.len()
        );

        // Initialise with average elevation
        let mut height_data = vec![average_elevation; TEXTURE_SIZE * TEXTURE_SIZE];
        let mut influence_map = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE];
        let mut road_height_map = vec![average_elevation; TEXTURE_SIZE * TEXTURE_SIZE];

        // Apply road heights with influence weighting
        for &(x, z, y) in road_points {
            let grid_x = (x * (TEXTURE_SIZE - 1) as f32) as i32;
            let grid_z = (z * (TEXTURE_SIZE - 1) as f32) as i32;

            self.apply_height_with_blend(
                &mut road_height_map,
                &mut influence_map,
                grid_x,
                grid_z,
                y,
            );
        }

        // Blend road heights with base elevation
        for i in 0..height_data.len() {
            let influence = influence_map[i].min(1.0);
            height_data[i] = average_elevation * (1.0 - influence) + road_height_map[i] * influence;
        }

        let heightmap_path = format!(
            "{}_heightmap_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        write_r32f_texture(&heightmap_path, TEXTURE_SIZE, &height_data)?;

        println!("Saved {} (R32F heightmap)", heightmap_path);
        Ok(())
    }

    /// Apply height value with gaussian blend in radius
    fn apply_height_with_blend(
        &self,
        road_height_map: &mut [f32],
        influence_map: &mut [f32],
        center_x: i32,
        center_z: i32,
        height: f32,
    ) {
        let radius = HEIGHTMAP_BLEND_RADIUS as i32;

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let px = center_x + dx;
                let pz = center_z + dz;

                if self.is_valid_coordinate(px, pz) {
                    let distance = ((dx * dx + dz * dz) as f32).sqrt();
                    if distance <= HEIGHTMAP_BLEND_RADIUS {
                        let pixel_index = (pz as usize) * TEXTURE_SIZE + (px as usize);
                        let influence = self.calculate_influence(distance);

                        self.apply_weighted_height(
                            road_height_map,
                            influence_map,
                            pixel_index,
                            height,
                            influence,
                        );
                    }
                }
            }
        }
    }

    /// Check if coordinates are within texture bounds
    fn is_valid_coordinate(&self, x: i32, z: i32) -> bool {
        x >= 0 && x < TEXTURE_SIZE as i32 && z >= 0 && z < TEXTURE_SIZE as i32
    }

    /// Calculate gaussian influence based on distance
    fn calculate_influence(&self, distance: f32) -> f32 {
        (-distance * distance / (HEIGHTMAP_BLEND_RADIUS * HEIGHTMAP_BLEND_RADIUS * 0.5)).exp()
    }

    /// Apply weighted height value using influence
    fn apply_weighted_height(
        &self,
        road_height_map: &mut [f32],
        influence_map: &mut [f32],
        pixel_index: usize,
        new_height: f32,
        influence: f32,
    ) {
        let current_influence = influence_map[pixel_index];
        let new_total_influence = current_influence + influence;

        if new_total_influence > 0.0 {
            road_height_map[pixel_index] = (road_height_map[pixel_index] * current_influence
                + new_height * influence)
                / new_total_influence;
            influence_map[pixel_index] = new_total_influence;
        }
    }
}
