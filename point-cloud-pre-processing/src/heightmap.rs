use crate::dds_writer::write_heightmap_dds;

pub struct HeightmapGenerator {
    output_stem: String,
}

impl HeightmapGenerator {
    pub fn new(output_stem: &str) -> Self {
        Self {
            output_stem: output_stem.to_string(),
        }
    }

    pub fn generate(
        &self,
        road_points: &[(f32, f32, f32)], // (x, z, y) normalized coordinates
        average_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        const HEIGHTMAP_SIZE: usize = 2048;
        const BLEND_RADIUS: f32 = 8.0;

        println!(
            "Creating {}x{} road heightmap from {} road points",
            HEIGHTMAP_SIZE,
            HEIGHTMAP_SIZE,
            road_points.len()
        );

        // Initialize heightmap with average elevation
        let mut height_data = vec![average_elevation; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE];
        let mut influence_map = vec![0.0f32; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE];
        let mut road_height_map = vec![average_elevation; HEIGHTMAP_SIZE * HEIGHTMAP_SIZE];

        // Apply road heights with influence weighting
        for &(x, z, y) in road_points {
            let grid_x = (x * (HEIGHTMAP_SIZE - 1) as f32) as i32;
            let grid_z = (z * (HEIGHTMAP_SIZE - 1) as f32) as i32;

            let radius = BLEND_RADIUS as i32;
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let px = grid_x + dx;
                    let pz = grid_z + dz;

                    if self.is_valid_coordinate(px, pz, HEIGHTMAP_SIZE) {
                        let distance = ((dx * dx + dz * dz) as f32).sqrt();
                        if distance <= BLEND_RADIUS {
                            let pixel_index = (pz as usize) * HEIGHTMAP_SIZE + (px as usize);

                            let influence = self.calculate_influence(distance, BLEND_RADIUS);
                            self.apply_weighted_height(
                                &mut road_height_map,
                                &mut influence_map,
                                pixel_index,
                                y,
                                influence,
                            );
                        }
                    }
                }
            }
        }

        // Blend road heights with base elevation
        for i in 0..height_data.len() {
            let influence = influence_map[i].min(1.0);
            height_data[i] = average_elevation * (1.0 - influence) + road_height_map[i] * influence;
        }

        println!("Applied road heights with smooth blending");

        let heightmap_path = format!("{}_road_heightmap.dds", self.output_stem);
        write_heightmap_dds(&heightmap_path, HEIGHTMAP_SIZE, &height_data)?;

        println!("Saved {} (R32_Float road heightmap)", heightmap_path);

        Ok(())
    }

    fn is_valid_coordinate(&self, x: i32, z: i32, size: usize) -> bool {
        x >= 0 && x < size as i32 && z >= 0 && z < size as i32
    }

    fn calculate_influence(&self, distance: f32, blend_radius: f32) -> f32 {
        (-distance * distance / (blend_radius * blend_radius * 0.5)).exp()
    }

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
