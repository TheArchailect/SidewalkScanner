/// Main point cloud converter orchestrating terrain and asset processing.
use crate::asset_processor::AssetProcessor;
use crate::atlas::generate_programmatic_name;
use crate::bounds::PointCloudBounds;
use crate::constants::{
    COLOUR_DETECTION_SAMPLE_SIZE, COORDINATE_TRANSFORM, MAX_POINTS, ROAD_CLASSIFICATIONS,
    TEXTURE_SIZE, get_class_name,
};
use crate::dds_writer::write_f32_texture;
use crate::heightmap::HeightmapGenerator;
use crate::manifest::{ClassificationInfo, ManifestGenerator, TerrainInfo, TerrainTextureFiles};
use crate::spatial_layout::SpatialTextureGenerator;
use indicatif::{ProgressBar, ProgressStyle};
use las::Reader;
use rayon::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Enhanced point cloud converter supporting terrain and asset processing.
/// Coordinates both legacy single-file and new asset library workflows.
pub struct PointCloudConverter {
    /// Primary terrain point cloud file path.
    main_cloud_path: PathBuf,
    /// Asset library directory containing .laz files.
    asset_library_dir: Option<PathBuf>,
    /// Output directory for preprocessed data.
    output_dir: PathBuf,
    /// Programmatic output name derived from input filename.
    output_name: String,
}

impl PointCloudConverter {
    /// Create new converter instance for single file processing.
    /// Maintains backward compatibility with existing workflows.
    pub fn new(input_path: &str, output_stem: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let main_cloud_path = PathBuf::from(input_path);
        let output_dir = main_cloud_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        Ok(Self {
            main_cloud_path,
            asset_library_dir: None,
            output_dir,
            output_name: generate_programmatic_name(output_stem),
        })
    }

    /// Creates converter instance with asset library support.
    /// Validates input paths and creates output directory structure.
    pub fn with_asset_library(
        main_cloud: &Path,
        asset_dir: &Path,
        output_dir: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if !main_cloud.exists() {
            return Err(format!("Main cloud file does not exist: {}", main_cloud.display()).into());
        }

        if !asset_dir.is_dir() {
            return Err(format!("Asset directory does not exist: {}", asset_dir.display()).into());
        }

        // Create organized output directory structure.
        fs::create_dir_all(output_dir)?;
        fs::create_dir_all(output_dir.join("terrain"))?;
        fs::create_dir_all(output_dir.join("assets"))?;

        // Generate programmatic name from main cloud filename.
        // Convert to owned String first to avoid borrowing temporary value.
        let filename = main_cloud
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let output_name = filename.trim_end_matches(".laz").trim_end_matches(".las");

        Ok(Self {
            main_cloud_path: main_cloud.to_path_buf(),
            asset_library_dir: Some(asset_dir.to_path_buf()),
            output_dir: output_dir.to_path_buf(),
            output_name: generate_programmatic_name(output_name),
        })
    }

    /// Executes complete preprocessing pipeline for terrain and assets.
    /// Generates both terrain textures and asset atlas with unified manifest.
    pub fn convert_with_assets(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting terrain and asset library processing...");

        // Process main terrain point cloud.
        let (terrain_info, classes) = self.process_main_terrain()?;

        // Process asset library if available.
        let asset_atlas_info = if let Some(asset_dir) = &self.asset_library_dir {
            let asset_processor = AssetProcessor::new(&self.output_dir, &self.output_name);
            Some(asset_processor.process_asset_library(asset_dir)?)
        } else {
            None
        };

        // Generate unified manifest linking terrain and assets.
        let manifest_gen = ManifestGenerator::new(&self.output_dir, &self.output_name);
        manifest_gen.generate_unified_manifest(terrain_info, asset_atlas_info, classes)?;

        println!("Asset library processing complete!");
        Ok(())
    }

    /// Legacy convert method for single file processing.
    /// Maintains compatibility with existing workflows.
    pub fn convert(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "Converting {} to unified texture set ({}x{})...",
            self.main_cloud_path.display(),
            TEXTURE_SIZE,
            TEXTURE_SIZE
        );

        self.log_file_info(&self.main_cloud_path)?;

        let has_colour = self.detect_colour_data(&self.main_cloud_path)?;
        let bounds = self.calculate_bounds(&self.main_cloud_path)?;
        self.print_bounds(&bounds);

        let (stats, road_points, classes) =
            self.generate_textures(&self.main_cloud_path, &bounds, has_colour)?;

        // Generate heightmap and save legacy metadata.
        self.generate_flood_fill_heightmap(&road_points)?;
        self.save_legacy_metadata(&bounds, &stats, has_colour)?;

        println!("Conversion complete!");
        Ok(())
    }

    /// Processes the main terrain point cloud for organized output.
    /// Returns terrain information for manifest generation.
    fn process_main_terrain(
        &self,
    ) -> Result<(TerrainInfo, ClassificationInfo), Box<dyn std::error::Error>> {
        println!(
            "Processing main terrain: {}",
            self.main_cloud_path.display()
        );

        self.log_file_info(&self.main_cloud_path)?;

        let has_colour = self.detect_colour_data(&self.main_cloud_path)?;
        let bounds = self.calculate_bounds(&self.main_cloud_path)?;
        self.print_bounds(&bounds);

        // Generate textures and save them to disk.
        let (stats, road_points, classes) =
            self.generate_textures(&self.main_cloud_path, &bounds, has_colour)?;

        // Generate heightmap using road surface points.
        self.generate_flood_fill_heightmap(&road_points)?;

        // Create organized terrain directory structure.
        let terrain_dir = self
            .output_dir
            .join("terrain")
            .join(format!("{:?}x{:?}", TEXTURE_SIZE, TEXTURE_SIZE));
        fs::create_dir_all(&terrain_dir)?;

        // Move terrain files to organized structure with programmatic names.
        self.organize_terrain_files(&terrain_dir)?;

        let texture_files = TerrainTextureFiles {
            position: format!(
                "terrain/{0}{1}x{1}/position.dds",
                self.output_name, TEXTURE_SIZE
            ),
            colour_class: format!(
                "terrain/{0}{1}x{1}/colourclass.dds",
                self.output_name, TEXTURE_SIZE
            ),
            spatial_index: format!(
                "terrain/{0}{1}x{1}/spatialindex.dds",
                self.output_name, TEXTURE_SIZE
            ),
            heightmap: format!(
                "terrain/{0}{1}x{1}/heightmap.dds",
                self.output_name, TEXTURE_SIZE
            ),
        };

        Ok((
            TerrainInfo {
                texture_files,
                bounds,
                point_count: stats.loaded_points,
                has_colour,
            },
            classes,
        ))
    }

    /// Organizes terrain files into programmatic directory structure.
    /// Moves generated textures from working directory to organized layout.
    fn organize_terrain_files(&self, terrain_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Define source and target file mappings with programmatic names.
        let file_mappings = [
            (
                format!(
                    "{}_position_{}x{}.dds",
                    self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
                ),
                "position.dds",
            ),
            (
                format!(
                    "{}_colour_class_{}x{}.dds",
                    self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
                ),
                "colourclass.dds",
            ),
            (
                format!(
                    "{}_spatial_index_{}x{}.dds",
                    self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
                ),
                "spatialindex.dds",
            ),
            (
                format!(
                    "{}_heightmap_{}x{}.dds",
                    self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
                ),
                "heightmap.dds",
            ),
        ];

        // Move files to organized structure.
        for (source_name, target_name) in &file_mappings {
            let source_path = self.output_dir.join(source_name);
            let target_path = terrain_dir.join(target_name);

            if source_path.exists() {
                fs::rename(&source_path, &target_path)?;
                println!("Organized: {} -> {}", source_name, target_path.display());
            }
        }

        Ok(())
    }

    /// Detect if colour data exists in the point cloud.
    /// Samples initial points to determine RGB availability for processing.
    fn detect_colour_data(&self, file_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
        let mut reader = self.create_reader(file_path)?;

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

    /// Calculate coordinate bounds from all points with parallel processing.
    /// Uses chunked parallel computation for efficient large dataset handling.
    fn calculate_bounds(
        &self,
        file_path: &Path,
    ) -> Result<PointCloudBounds, Box<dyn std::error::Error>> {
        let mut reader = self.create_reader(file_path)?;
        let total_points = reader.header().number_of_points() as usize;

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

    /// Generate spatial textures using Z-order layout and sampling.
    /// Returns processing statistics and road points for heightmap generation.
    fn generate_textures(
        &self,
        file_path: &Path,
        bounds: &PointCloudBounds,
        has_colour: bool,
    ) -> Result<
        (ProcessingStats, Vec<(f32, f32, f32)>, ClassificationInfo),
        Box<dyn std::error::Error>,
    > {
        let mut reader = self.create_reader(file_path)?;
        let total_points = reader.header().number_of_points() as usize;

        let sampling_ratio = if total_points > MAX_POINTS {
            MAX_POINTS as f64 / total_points as f64
        } else {
            1.0
        };

        // Create spatial generator with n*n grid for Z-order organization.
        let mut spatial_gen = SpatialTextureGenerator::new(bounds.clone(), 1024);
        let mut road_points = Vec::new();
        let mut stats = ProcessingStats::new();
        let mut expected_loaded = 0.0;

        println!(
            "Sampling ratio: {:.3} ({:.1}% of points)",
            sampling_ratio,
            sampling_ratio * 100.0
        );

        // Process points with progress tracking and spatial organization.
        let pb = ProgressBar::new(total_points as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} points ({percent}%) {msg}")
                .unwrap()
                .progress_chars("▉▊▋▌▍▎▏ "),
        );
        pb.set_message("Processing points spatially");

        let mut classes = ClassificationInfo {
            class_types: HashMap::new(),
        };

        for (point_idx, point_result) in reader.points().enumerate() {
            expected_loaded += sampling_ratio;

            if point_idx % 10_000 == 0 {
                pb.set_position(point_idx as u64);
            }

            // Apply sampling to stay within texture limits.
            if stats.loaded_points as f64 >= expected_loaded || stats.loaded_points >= MAX_POINTS {
                continue;
            }

            let point = point_result?;
            let (x, y, z) = self.transform_coordinates(point.x, point.y, point.z);
            let classification = u8::from(point.classification);
            let color = point.color.map(|c| (c.red, c.green, c.blue));

            // Extract object number from extra bytes if available.
            let object_number = if point.extra_bytes.len() >= 4 {
                f32::from_le_bytes([
                    point.extra_bytes[0],
                    point.extra_bytes[1],
                    point.extra_bytes[2],
                    point.extra_bytes[3],
                ])
            } else {
                0.0
            };

            // Store unique point class combinations in our class info struct
            classes.insert_or_update(
                classification,
                get_class_name(classification),
                object_number as u32,
            );

            // Add to spatial structure for Z-order organization.
            spatial_gen.add_point((x, y, z), classification, color, object_number);

            // Track road points for heightmap generation.
            if ROAD_CLASSIFICATIONS.contains(&classification) {
                let norm_x = bounds.normalize_x(x);
                let norm_z = bounds.normalize_z(z);
                let norm_y = bounds.normalize_y(y);
                road_points.push((norm_x, norm_z, norm_y));
            }

            // Update processing statistics.
            if color.is_some() {
                stats.colour_points += 1;
            }
            stats.total_elevation += y;
            stats.loaded_points += 1;
        }

        pb.finish_with_message("Points processed");

        println!("Found Class Info: {:?}", classes);

        // Apply spatial sorting and generate textures.
        println!("Applying Z-order spatial sorting...");
        spatial_gen.sort_spatially();

        let position_data = spatial_gen.generate_position_texture();
        let colour_class_data = spatial_gen.generate_colour_class_texture();
        let spatial_index_data = spatial_gen.generate_spatial_index_texture();

        self.print_processing_stats(&stats, total_points, has_colour);

        // Save generated textures with programmatic names.
        self.save_textures(&position_data, &colour_class_data, &spatial_index_data)?;

        Ok((stats, road_points, classes))
    }

    /// Generate flood-fill heightmap from road surface points.
    /// Uses HeightmapGenerator for smooth terrain surface reconstruction.
    fn generate_flood_fill_heightmap(
        &self,
        road_points: &[(f32, f32, f32)],
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create heightmap generator with output directory context.
        let heightmap_gen = HeightmapGenerator::new(&self.output_name);

        // Generate heightmap in the correct output directory.
        let original_dir = std::env::current_dir()?;
        std::env::set_current_dir(&self.output_dir)?;

        let result = heightmap_gen.generate_flood_fill_from_road_points(road_points);

        // Restore original directory.
        std::env::set_current_dir(original_dir)?;

        result
    }

    /// Apply coordinate transformation matrix to ensure consistency.
    /// Transforms input coordinates using predefined transformation matrix.
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

    /// Save position, color+classification, and spatial index textures.
    /// Uses programmatic naming convention for organized output.
    fn save_textures(
        &self,
        position_data: &[f32],
        colour_class_data: &[f32],
        spatial_index_data: &[f32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pos_path = self.output_dir.join(format!(
            "{}_position_{}x{}.dds",
            self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
        ));
        let colour_path = self.output_dir.join(format!(
            "{}_colour_class_{}x{}.dds",
            self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
        ));
        let spatial_path = self.output_dir.join(format!(
            "{}_spatial_index_{}x{}.dds",
            self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
        ));

        write_f32_texture(
            pos_path.to_str().unwrap(),
            TEXTURE_SIZE,
            position_data,
            ddsfile::DxgiFormat::R32G32B32A32_Float,
        )?;
        write_f32_texture(
            colour_path.to_str().unwrap(),
            TEXTURE_SIZE,
            colour_class_data,
            ddsfile::DxgiFormat::R32G32B32A32_Float,
        )?;
        write_f32_texture(
            spatial_path.to_str().unwrap(),
            TEXTURE_SIZE,
            spatial_index_data,
            ddsfile::DxgiFormat::R32G32_Float,
        )?;

        println!("Saved {} (Position RGBA32F)", pos_path.display());
        println!("Saved {} (Colour+Class RGBA32F)", colour_path.display());
        println!("Saved {} (Spatial Index RGBA32F)", spatial_path.display());

        Ok(())
    }

    /// Save processing metadata as JSON for legacy single-file workflow.
    /// Maintains backward compatibility with existing metadata format.
    fn save_legacy_metadata(
        &self,
        bounds: &PointCloudBounds,
        stats: &ProcessingStats,
        has_colour: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = serde_json::json!({
            "texture_size": TEXTURE_SIZE,
            "total_points": stats.loaded_points, // Note: this was inconsistent in original
            "loaded_points": stats.loaded_points,
            "utilisation_percent": (stats.loaded_points as f32 / MAX_POINTS as f32) * 100.0,
            "sampling_ratio": if stats.loaded_points > 0 {
                stats.loaded_points as f64 / MAX_POINTS as f64
            } else { 0.0 },
            "has_colour": has_colour,
            "colour_points": stats.colour_points,
            "bounds": {
                "min_x": bounds.min_x, "max_x": bounds.max_x,
                "min_y": bounds.min_y, "max_y": bounds.max_y,
                "min_z": bounds.min_z, "max_z": bounds.max_z
            },
            "textures": {
                "position": "RGBA32F - XYZ coordinates + object number",
                "colour_class": "RGBA32F - RGB colour + classification",
                "spatial_index": "RGBA32F - Morton codes + spatial data",
                "heightmap": format!("R32F - road surface elevation {}x{}", TEXTURE_SIZE, TEXTURE_SIZE)
            }
        });

        let metadata_path = format!(
            "{}_metadata_{}x{}.json",
            self.output_name, TEXTURE_SIZE, TEXTURE_SIZE
        );
        std::fs::write(&metadata_path, metadata.to_string())?;
        println!("Saved {}", metadata_path);

        Ok(())
    }

    /// Create LAS file reader for point cloud access.
    /// Handles both .las and .laz compressed formats.
    fn create_reader(&self, file_path: &Path) -> Result<Reader, Box<dyn std::error::Error>> {
        let file = File::open(file_path)?;
        let buf_reader = BufReader::new(file);
        Ok(Reader::new(buf_reader)?)
    }

    /// Print coordinate bounds information for validation.
    /// Displays transformed bounds for debugging and verification.
    fn print_bounds(&self, bounds: &PointCloudBounds) {
        println!("Transformed bounds:");
        println!("  X: {:.2} to {:.2}", bounds.min_x, bounds.max_x);
        println!(
            "  Y: {:.2} to {:.2} (elevation)",
            bounds.min_y, bounds.max_y
        );
        println!("  Z: {:.2} to {:.2} (depth)", bounds.min_z, bounds.max_z);
    }

    /// Print processing statistics for verification and debugging.
    /// Shows texture utilization, sampling efficiency, and color data availability.
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

    /// Log coordinate system and file information for debugging.
    /// Provides detailed information about LAS file structure and coordinate systems.
    fn log_file_info(&self, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let reader = self.create_reader(file_path)?;
        let header = reader.header();

        println!("LAS/LAZ File Information:");
        println!("  File: {}", file_path.display());
        println!(
            "  Version: {}.{}",
            header.version().major,
            header.version().minor
        );
        println!("  Points: {}", header.number_of_points());
        println!("  Point format: {:?}", header.point_format().to_u8());

        // Display coordinate system information.
        println!("  Coordinate System:");
        let x_scale = header.transforms().x.scale;
        let y_scale = header.transforms().y.scale;
        let z_scale = header.transforms().z.scale;

        println!(
            "    Scale factors: X={}, Y={}, Z={}",
            x_scale, y_scale, z_scale
        );

        let x_offset = header.transforms().x.offset;
        let y_offset = header.transforms().y.offset;
        let z_offset = header.transforms().z.offset;

        println!(
            "    Offsets: X={}, Y={}, Z={}",
            x_offset, y_offset, z_offset
        );

        // Check for extra bytes and coordinate reference system data.
        for vlr in header.vlrs() {
            if vlr.record_id == 4 {
                println!("    Extra Bytes VLR found: {} bytes", vlr.data.len());
                if vlr.data.len() >= 17 {
                    let field_name = String::from_utf8_lossy(&vlr.data[4..17]);
                    let field_name = field_name.trim_end_matches('\0');
                    let data_type = vlr.data[2];
                    println!("    Field name: '{}'", field_name);
                    println!("    Data type: {} (9=f32)", data_type);
                }
            }
        }

        println!();
        Ok(())
    }
}

/// Processing statistics tracker for monitoring conversion progress.
/// Tracks point counts, color availability, and elevation data.
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
