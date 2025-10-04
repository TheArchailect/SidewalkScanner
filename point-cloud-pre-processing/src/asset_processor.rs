/// Asset library processing module for atlas generation.
use crate::atlas::{
    AssetAtlasInfo, AssetCandidate, AssetMetadata, AtlasConfig, AtlasTextureFiles,
    AtlasTextureGenerator, discover_asset_files,
};
use crate::bounds::PointCloudBounds;
use crate::constants::{COORDINATE_TRANSFORM, TEXTURE_SIZE};
use crate::dds_writer::write_f32_texture;
use crate::spatial_layout::SpatialPoint;
use indicatif::{ProgressBar, ProgressStyle};
use las::Reader;
use serde_json::json;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

/// Asset library processor for generating texture atlases.
/// Handles discovery, validation, and atlas generation for GPU rendering.
pub struct AssetProcessor {
    /// Output directory for organized asset files.
    output_dir: std::path::PathBuf,
    /// Base name for generated files.
    output_name: String,
}

impl AssetProcessor {
    /// Creates new asset processor with output configuration.
    /// Sets up directory structure for atlas organization.
    pub fn new(output_dir: &Path, output_name: &str) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            output_name: output_name.to_string(),
        }
    }

    /// Processes asset library and generates atlas textures.
    /// Returns asset atlas information for manifest integration.
    pub fn process_asset_library(
        &self,
        asset_dir: &Path,
    ) -> Result<AssetAtlasInfo, Box<dyn std::error::Error>> {
        println!("Processing asset library: {}", asset_dir.display());

        // Discover and validate asset files.
        let candidates = discover_asset_files(asset_dir)?;
        println!("Found {} asset candidates", candidates.len());

        if candidates.is_empty() {
            return Err("No valid asset files found in library directory".into());
        }

        let mut atlas_gen = AtlasTextureGenerator::new();

        // Process each asset with progress tracking.
        let pb = ProgressBar::new(candidates.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.green/blue}] {pos}/{len} assets ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏"),
        );
        pb.set_message("Processing assets");

        for (idx, candidate) in candidates.iter().enumerate() {
            println!(
                "Processing #{}: {} -> will assign to tile",
                idx, candidate.name
            );
            let (asset_points, local_bounds) = self.load_asset_points(&candidate)?;
            atlas_gen.add_asset(&asset_points, candidate.name.clone(), local_bounds)?;
            pb.inc(1);
        }

        pb.finish_with_message("Assets processed");

        // Generate and save atlas textures.
        self.generate_and_save_atlas(&atlas_gen)?;

        // Build atlas info for manifest.
        let texture_files = AtlasTextureFiles {
            position: format!(
                "assets/AssetAtlas{}x{}/position.dds",
                TEXTURE_SIZE, TEXTURE_SIZE
            ),
            colour_class: format!(
                "assets/AssetAtlas{}x{}/colourclass.dds",
                TEXTURE_SIZE, TEXTURE_SIZE
            ),
        };

        Ok(AssetAtlasInfo {
            texture_files,
            assets: atlas_gen.get_asset_metadata().to_vec(),
            atlas_config: atlas_gen.get_config().clone(),
        })
    }

    /// Generates atlas textures and saves to organized directory structure.
    /// Creates both position and color/classification atlases.
    fn generate_and_save_atlas(
        &self,
        atlas_gen: &AtlasTextureGenerator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating atlas textures...");

        let atlas_textures = atlas_gen.generate_atlas_textures();

        // Create asset directory structure.
        let asset_dir_path = self
            .output_dir
            .join("assets")
            .join("AssetAtlas")
            .join(format!("{}x{}", TEXTURE_SIZE, TEXTURE_SIZE));
        fs::create_dir_all(&asset_dir_path)?;

        // Save atlas textures with organized naming.
        let position_path = asset_dir_path.join("position.dds");
        let colour_class_path = asset_dir_path.join("colourclass.dds");

        write_f32_texture(
            position_path.to_str().unwrap(),
            TEXTURE_SIZE,
            &atlas_textures.position_data,
            ddsfile::DxgiFormat::R32G32B32A32_Float,
        )?;

        write_f32_texture(
            colour_class_path.to_str().unwrap(),
            TEXTURE_SIZE,
            &atlas_textures.colour_class_data,
            ddsfile::DxgiFormat::R32G32B32A32_Float,
        )?;

        println!("Saved asset atlas textures");

        // Save atlas metadata for debugging and validation.
        self.save_atlas_metadata(&asset_dir_path, atlas_gen)?;

        Ok(())
    }

    /// Saves atlas metadata including configuration and asset details.
    /// Provides debugging information and validation data.
    fn save_atlas_metadata(
        &self,
        asset_dir: &Path,
        atlas_gen: &AtlasTextureGenerator,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = json!({
            "atlas_config": atlas_gen.get_config(),
            "assets": atlas_gen.get_asset_metadata(),
            "texture_format": "RGBA32F",
            "coordinate_transform": COORDINATE_TRANSFORM
        });

        let metadata_path = asset_dir.join("atlas_metadata.json");
        fs::write(&metadata_path, metadata.to_string())?;
        println!("Saved atlas metadata: {}", metadata_path.display());

        Ok(())
    }

    /// Loads and processes points from an asset file.
    /// Returns spatial points and local bounds for atlas integration.
    fn load_asset_points(
        &self,
        candidate: &AssetCandidate,
    ) -> Result<(Vec<SpatialPoint>, PointCloudBounds), Box<dyn std::error::Error>> {
        let file = File::open(&candidate.path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::new(buf_reader)?;

        let mut asset_points = Vec::new();
        let mut local_bounds = PointCloudBounds::new();

        // Load all points from asset file.
        for point_result in reader.points() {
            let point = point_result?;
            let (x, y, z) = self.transform_coordinates(point.x, point.y, point.z);

            // Update local bounds for this asset.
            local_bounds.update(x, y, z);

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

            // Create spatial point with asset-local coordinates.
            let spatial_point = SpatialPoint {
                world_pos: (x, y, z),
                norm_pos: (
                    local_bounds.normalize_x(x),
                    local_bounds.normalize_y(y),
                    local_bounds.normalize_z(z),
                ),
                morton_index: 0,    // Will be calculated if needed.
                spatial_cell_id: 0, // Not used for asset atlas.
                classification,
                color,
                object_number,
            };

            asset_points.push(spatial_point);
        }

        // Normalize coordinates after loading all points.
        for point in &mut asset_points {
            point.norm_pos = (
                local_bounds.normalize_x(point.world_pos.0),
                local_bounds.normalize_y(point.world_pos.1),
                local_bounds.normalize_z(point.world_pos.2),
            );
        }

        println!(
            "Loaded {} points from {}",
            asset_points.len(),
            candidate.name
        );
        Ok((asset_points, local_bounds))
    }

    /// Apply coordinate transformation matrix to point coordinates.
    /// Ensures consistency with main terrain coordinate system.
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
}
