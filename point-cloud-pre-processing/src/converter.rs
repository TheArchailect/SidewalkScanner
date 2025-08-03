use crate::bounds::PointCloudBounds;
use crate::dds_writer::{write_metadata_dds, write_position_dds};
use crate::heightmap::HeightmapGenerator;
use half::f16;
use las::Reader;
use serde_json;
use std::fs::File;
use std::io::BufReader;

pub struct PointCloudConverter {
    input_path: String,
    output_stem: String,
}

impl PointCloudConverter {
    pub fn new(input_path: &str, output_stem: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            input_path: input_path.to_string(),
            output_stem: output_stem.to_string(),
        })
    }

    pub fn convert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // First pass: Calculate bounds
        println!("Pass 1: Calculating bounds...");
        let bounds = self.calculate_bounds()?;
        println!("Bounds: {:?}", bounds);

        // Second pass: Generate textures
        println!("Pass 2: Generating DDS textures...");
        self.generate_dds_textures(&bounds)?;

        Ok(())
    }

    fn calculate_bounds(&self) -> Result<PointCloudBounds, Box<dyn std::error::Error>> {
        let file = File::open(&self.input_path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::new(buf_reader)?;

        let total_points = reader.header().number_of_points() as usize;
        println!("Total points in file: {}", total_points);

        let mut bounds = PointCloudBounds::new();
        let mut processed = 0;

        for point_result in reader.points() {
            let point = point_result?;

            // Apply coordinate transformation: Rotate -90° around X axis
            let transformed_x = point.x; // X stays X
            let transformed_y = point.z; // Z becomes Y (up)
            let transformed_z = -point.y; // -Y becomes Z (forward)

            bounds.update(transformed_x, transformed_y, transformed_z);

            processed += 1;
            if processed % 1_000_000 == 0 {
                println!("  Processed {} / {} points", processed, total_points);
            }
        }

        println!("Transformed bounds (after -90° X rotation):");
        println!("  X: {:.2} to {:.2}", bounds.min_x, bounds.max_x);
        println!(
            "  Y: {:.2} to {:.2} (was Z elevation)",
            bounds.min_y, bounds.max_y
        );
        println!(
            "  Z: {:.2} to {:.2} (was -Y northing)",
            bounds.min_z, bounds.max_z
        );

        Ok(bounds)
    }

    fn generate_dds_textures(
        &self,
        bounds: &PointCloudBounds,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(&self.input_path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::new(buf_reader)?;

        let total_points = reader.header().number_of_points() as usize;

        const TEXTURE_SIZE: usize = 8192;
        const MAX_POINTS: usize = TEXTURE_SIZE * TEXTURE_SIZE;

        println!("File has {} points", total_points);
        println!("8K texture can hold {} points", MAX_POINTS);

        let sampling_ratio = if total_points > MAX_POINTS {
            MAX_POINTS as f64 / total_points as f64
        } else {
            1.0
        };

        println!(
            "Sampling ratio: {:.3} ({:.1}% of points)",
            sampling_ratio,
            sampling_ratio * 100.0
        );

        // Initialize data arrays
        let mut position_data = vec![f16::ZERO; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        let mut metadata_data = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE * 4];
        let mut road_points = Vec::<(f32, f32, f32)>::new();
        let mut total_elevation = 0.0;
        let mut elevation_count = 0;
        let mut loaded_points = 0;
        let mut expected_loaded = 0.0;

        println!("Processing points with uniform sampling...");

        for point_result in reader.points() {
            expected_loaded += sampling_ratio;
            if loaded_points as f64 >= expected_loaded {
                continue;
            }

            if loaded_points >= MAX_POINTS {
                println!("Texture full at {} points", loaded_points);
                break;
            }

            let point = point_result?;

            // Apply coordinate transformation
            let transformed_x = point.x;
            let transformed_y = point.z;
            let transformed_z = -point.y;

            let pixel_index = loaded_points;
            let pos_offset = pixel_index * 4;
            let meta_offset = pixel_index * 4;

            // Normalize coordinates to 0-1 range
            let norm_x = bounds.normalize_x(transformed_x);
            let norm_y = bounds.normalize_y(transformed_y);
            let norm_z = bounds.normalize_z(transformed_z);

            // Position texture: 16-bit float precision
            position_data[pos_offset] = f16::from_f32(norm_x);
            position_data[pos_offset + 1] = f16::from_f32(norm_y);
            position_data[pos_offset + 2] = f16::from_f32(norm_z);
            position_data[pos_offset + 3] = f16::ONE; // Valid point marker

            // Metadata texture
            let classification = u8::from(point.classification);
            let intensity = (point.intensity >> 8) as u8;
            let classification_f = classification as f32 / 255.0;
            let intensity_f = intensity as f32 / 255.0;
            let return_info =
                ((point.return_number & 0x0F) << 4) | (point.number_of_returns & 0x0F);

            metadata_data[meta_offset] = classification_f;
            metadata_data[meta_offset + 1] = intensity_f;
            metadata_data[meta_offset + 2] = return_info as f32 / 255.0;
            metadata_data[meta_offset + 3] = 1.0;

            if [2, 10, 11, 12].contains(&classification) {
                road_points.push((norm_x, norm_z, norm_y));
            }
            total_elevation += norm_y as f64;
            elevation_count += 1;

            loaded_points += 1;

            if loaded_points % 1_000_000 == 0 {
                println!("  Loaded {} points", loaded_points);
            }

            if loaded_points % 500_000 == 0 {
                println!(
                    "Point {}: class={}, stored={}",
                    loaded_points, classification, metadata_data[meta_offset]
                );
            }
        }

        println!("Final stats:");
        println!("  Actually loaded: {}", loaded_points);
        println!(
            "  Texture utilization: {:.1}%",
            (loaded_points as f32 / MAX_POINTS as f32) * 100.0
        );

        // Save DDS files
        let pos_path = format!("{}_positions.dds", self.output_stem);
        let meta_path = format!("{}_metadata.dds", self.output_stem);
        let bounds_path = format!("{}_bounds.json", self.output_stem);

        write_position_dds(&pos_path, TEXTURE_SIZE, &position_data)?;
        write_metadata_dds(&meta_path, TEXTURE_SIZE, &metadata_data)?;

        // Save bounds JSON
        let bounds_json = serde_json::json!({
            "min_x": bounds.min_x,
            "max_x": bounds.max_x,
            "min_y": bounds.min_y,
            "max_y": bounds.max_y,
            "min_z": bounds.min_z,
            "max_z": bounds.max_z,
            "total_points": total_points,
            "loaded_points": loaded_points,
            "texture_size": TEXTURE_SIZE,
            "sampling_ratio": sampling_ratio,
            "utilization_percent": (loaded_points as f32 / MAX_POINTS as f32) * 100.0
        });

        std::fs::write(&bounds_path, bounds_json.to_string())?;

        println!("Saved {} (16-bit float, BC6H-ready)", pos_path);
        println!("Saved {} (8-bit optimized)", meta_path);
        println!("Saved {}", bounds_path);

        // Generate road heightmap
        let average_elevation = if elevation_count > 0 {
            (total_elevation / elevation_count as f64) as f32
        } else {
            0.5
        };
        println!("Average elevation: {:.3} (normalized)", average_elevation);
        println!("Generating road heightmap...");

        let heightmap_gen = HeightmapGenerator::new(&self.output_stem);
        heightmap_gen.generate(&road_points, average_elevation)?;

        Ok(())
    }
}
