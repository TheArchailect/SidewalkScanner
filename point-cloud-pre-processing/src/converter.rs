/// Unified point cloud to texture converter
use crate::bounds::PointCloudBounds;
use crate::constants::{
    COLOUR_DETECTION_SAMPLE_SIZE, COORDINATE_TRANSFORM, MAX_POINTS, ROAD_CLASSIFICATIONS,
    TEXTURE_SIZE,
};
use crate::dds_writer::{write_r32f_texture, write_rgba32f_texture};
use crate::heightmap::HeightmapGenerator;
use indicatif::{ProgressBar, ProgressStyle};
use las::Reader;
use rayon::prelude::*;
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Point cloud to texture converter with unified pipeline
pub struct PointCloudConverter {
    input_path: String,
    output_stem: String,
}

impl PointCloudConverter {
    /// Create new converter instance
    pub fn new(input_path: &str, output_stem: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            input_path: input_path.to_string(),
            output_stem: output_stem.to_string(),
        })
    }

    /// Convert LAZ/LAS file to unified texture set
    pub fn convert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Converting {} to unified texture set ({}x{})...",
            self.input_path, TEXTURE_SIZE, TEXTURE_SIZE
        );

        let has_colour = self.detect_colour_data()?;
        let bounds = self.calculate_bounds()?;
        self.print_bounds(&bounds);
        self.generate_textures(&bounds, has_colour)?;

        println!("Conversion complete!");
        Ok(())
    }

    /// Detect if colour data exists in the point cloud
    fn detect_colour_data(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut reader = self.create_reader()?;

        let mut colour_count = 0;
        let mut total_checked = 0;

        for point_result in reader.points().take(COLOUR_DETECTION_SAMPLE_SIZE) {
            if let Ok(point) = point_result {
                if point.color.is_some() {
                    colour_count += 1;
                }
                total_checked += 1;
            }
        }

        let has_colour = colour_count > 0;
        if has_colour {
            println!(
                "Colour data detected: {}/{} sample points have RGB",
                colour_count, total_checked
            );
        } else {
            println!("No colour data found");
        }

        Ok(has_colour)
    }

    /// Calculate coordinate bounds from all points (parallel with progress)
    fn calculate_bounds(&self) -> Result<PointCloudBounds, Box<dyn std::error::Error>> {
        let mut reader = self.create_reader()?;
        let total_points = reader.header().number_of_points() as usize;

        // Progress bar for loading
        let pb = ProgressBar::new(total_points as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos}/{len} points ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Loading points");

        // Collect points with progress updates
        let mut all_points = Vec::with_capacity(total_points);
        for (idx, point_result) in reader.points().enumerate() {
            all_points.push(point_result?);

            if idx % 50_000 == 0 {
                pb.set_position(idx as u64);
            }
        }
        pb.finish_with_message("Points loaded");

        // Setup bounds calculation progress
        let pb = ProgressBar::new(all_points.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} chunks ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Calculating bounds");

        // Process bounds in parallel chunks
        let bounds = all_points
            .par_chunks(25_000)
            .map(|chunk| {
                let mut local_bounds = PointCloudBounds::new();
                for point in chunk {
                    let (x, y, z) = self.transform_coordinates(point.x, point.y, point.z);
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
            .unwrap_or_else(PointCloudBounds::new);

        pb.finish_with_message("Bounds calculated");
        Ok(bounds)
    }

    /// Generate all texture outputs
    fn generate_textures(
        &self,
        bounds: &PointCloudBounds,
        has_colour: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = self.create_reader()?;
        let total_points = reader.header().number_of_points() as usize;

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

        // Progress bar for texture generation
        let pb = ProgressBar::new(total_points as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} points ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Generating textures");

        // Initialise texture data arrays
        let mut position_data = vec![0.0f32; MAX_POINTS * 4];
        let mut colour_class_data = vec![0.0f32; MAX_POINTS * 4];
        let mut road_points = Vec::new();

        let mut stats = ProcessingStats::new();
        let mut expected_loaded = 0.0;

        // Process points with uniform sampling
        for (point_idx, point_result) in reader.points().enumerate() {
            expected_loaded += sampling_ratio;

            // Update progress every 10k points
            if point_idx % 10_000 == 0 {
                pb.set_position(point_idx as u64);
            }

            if stats.loaded_points as f64 >= expected_loaded || stats.loaded_points >= MAX_POINTS {
                continue;
            }

            let point = point_result?;
            let (x, y, z) = self.transform_coordinates(point.x, point.y, point.z);

            // Normalise coordinates
            let norm_x = bounds.normalize_x(x);
            let norm_y = bounds.normalize_y(y);
            let norm_z = bounds.normalize_z(z);

            let offset = stats.loaded_points * 4;

            // Position texture: RGBA32F (X, Y, Z, validity)
            position_data[offset] = norm_x;
            position_data[offset + 1] = norm_y;
            position_data[offset + 2] = norm_z;
            position_data[offset + 3] = 1.0; // Valid point marker

            // Colour + Classification texture: RGBA32F (R, G, B, classification)
            if let Some(color) = point.color {
                colour_class_data[offset] = color.red as f32 / 65535.0;
                colour_class_data[offset + 1] = color.green as f32 / 65535.0;
                colour_class_data[offset + 2] = color.blue as f32 / 65535.0;
                stats.colour_points += 1;
            } else {
                // Default white for points without colour
                colour_class_data[offset] = 1.0;
                colour_class_data[offset + 1] = 1.0;
                colour_class_data[offset + 2] = 1.0;
            }
            colour_class_data[offset + 3] = u8::from(point.classification) as f32 / 255.0;

            // Track road points for heightmap
            let classification = u8::from(point.classification);
            if ROAD_CLASSIFICATIONS.contains(&classification) {
                road_points.push((norm_x, norm_z, norm_y));
            }

            stats.total_elevation += norm_y as f64;
            stats.loaded_points += 1;
        }

        pb.finish_with_message("Textures generated");
        self.print_processing_stats(&stats, total_points, has_colour);

        // Save textures
        self.save_textures(&position_data, &colour_class_data)?;

        // Generate heightmap
        let avg_elevation = (stats.total_elevation / stats.loaded_points as f64) as f32;
        self.generate_heightmap(&road_points, avg_elevation)?;

        // Save metadata
        self.save_metadata(bounds, &stats, total_points, sampling_ratio, has_colour)?;

        Ok(())
    }

    /// Apply coordinate transformation matrix
    fn transform_coordinates(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let input = [x, y, z];
        let mut output = [0.0; 3];

        for i in 0..3 {
            for j in 0..3 {
                output[i] += COORDINATE_TRANSFORM[i][j] * input[j];
            }
        }

        (output[0], output[1], output[2])
    }

    /// Save position and colour+classification textures
    fn save_textures(
        &self,
        position_data: &[f32],
        colour_class_data: &[f32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pos_path = format!(
            "{}_position_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        let colour_path = format!(
            "{}_colour_class_{}x{}.dds",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );

        write_rgba32f_texture(&pos_path, TEXTURE_SIZE, position_data)?;
        write_rgba32f_texture(&colour_path, TEXTURE_SIZE, colour_class_data)?;

        println!("Saved {} (Position RGBA32F)", pos_path);
        println!("Saved {} (Colour+Class RGBA32F)", colour_path);

        Ok(())
    }

    /// Generate road surface heightmap
    fn generate_heightmap(
        &self,
        road_points: &[(f32, f32, f32)],
        avg_elevation: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Generating road heightmap from {} road points...",
            road_points.len()
        );

        let heightmap_gen = HeightmapGenerator::new(&self.output_stem);
        heightmap_gen.generate_unified(road_points, avg_elevation)?;

        Ok(())
    }

    /// Save processing metadata as JSON
    fn save_metadata(
        &self,
        bounds: &PointCloudBounds,
        stats: &ProcessingStats,
        total_points: usize,
        sampling_ratio: f64,
        has_colour: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = serde_json::json!({
            "texture_size": TEXTURE_SIZE,
            "total_points": total_points,
            "loaded_points": stats.loaded_points,
            "utilisation_percent": (stats.loaded_points as f32 / MAX_POINTS as f32) * 100.0,
            "sampling_ratio": sampling_ratio,
            "has_colour": has_colour,
            "colour_points": stats.colour_points,
            "bounds": {
                "min_x": bounds.min_x, "max_x": bounds.max_x,
                "min_y": bounds.min_y, "max_y": bounds.max_y,
                "min_z": bounds.min_z, "max_z": bounds.max_z
            },
            "textures": {
                "position": "RGBA32F - XYZ coordinates + validity",
                "colour_class": "RGBA32F - RGB colour + classification",
                "heightmap": format!("R32F - road surface elevation {}x{}", TEXTURE_SIZE, TEXTURE_SIZE)
            }
        });

        let metadata_path = format!(
            "{}_metadata_{}x{}.json",
            self.output_stem, TEXTURE_SIZE, TEXTURE_SIZE
        );
        std::fs::write(&metadata_path, metadata.to_string())?;
        println!("Saved {}", metadata_path);

        Ok(())
    }

    /// Create LAS file reader
    fn create_reader(&self) -> Result<Reader, Box<dyn std::error::Error>> {
        let file = File::open(&self.input_path)?;
        let buf_reader = BufReader::new(file);
        Ok(Reader::new(buf_reader)?)
    }

    /// Print coordinate bounds information
    fn print_bounds(&self, bounds: &PointCloudBounds) {
        println!("Transformed bounds:");
        println!("  X: {:.2} to {:.2}", bounds.min_x, bounds.max_x);
        println!(
            "  Y: {:.2} to {:.2} (elevation)",
            bounds.min_y, bounds.max_y
        );
        println!("  Z: {:.2} to {:.2} (depth)", bounds.min_z, bounds.max_z);
    }

    /// Print processing statistics
    fn print_processing_stats(
        &self,
        stats: &ProcessingStats,
        total_points: usize,
        has_colour: bool,
    ) {
        println!("Processing complete:");
        println!(
            "  Loaded: {} points ({:.1}% texture utilisation)",
            stats.loaded_points,
            (stats.loaded_points as f32 / MAX_POINTS as f32) * 100.0
        );

        if has_colour {
            println!(
                "  Colour points: {} ({:.1}%)",
                stats.colour_points,
                (stats.colour_points as f32 / stats.loaded_points as f32) * 100.0
            );
        }
    }
}

/// Processing statistics tracker
struct ProcessingStats {
    loaded_points: usize,
    colour_points: usize,
    total_elevation: f64,
}

impl ProcessingStats {
    fn new() -> Self {
        Self {
            loaded_points: 0,
            colour_points: 0,
            total_elevation: 0.0,
        }
    }
}
