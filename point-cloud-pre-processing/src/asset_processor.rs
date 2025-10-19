/// Asset library processing module for atlas generation.
use crate::atlas::{
    AssetAtlasInfo, AssetCandidate, AtlasTextureFiles, AtlasTextureGenerator, discover_asset_files,
};
use crate::bounds::PointCloudBounds;
use crate::bounds::calculate_bounds;
use crate::dds_writer::write_f32_texture;
use crate::spatial_layout::SpatialPoint;
use constants::coordinate_system::transform_coordinates;
use constants::texture::TEXTURE_SIZE;
use indicatif::{ProgressBar, ProgressStyle};
use las::Reader;
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
    /// Sets up directory structure for atlas organisation.
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
        let candidates = discover_asset_files(asset_dir)?;
        println!("Found {} asset candidates", candidates.len());
        if candidates.is_empty() {
            return Err("No valid asset files found in library directory".into());
        }

        let mut atlas_gen = AtlasTextureGenerator::new();
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
            let (asset_points, asset_bound) = self.load_asset_points(&candidate)?;
            atlas_gen.add_asset(&asset_points, candidate.name.clone(), asset_bound)?;
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
        Ok(())
    }

    /// Loads and processes points from an asset file with (match size) based on full local bound
    /// Returns worldspace points and local bounds for atlas integration.
    fn load_asset_points(
        &self,
        candidate: &AssetCandidate,
    ) -> Result<(Vec<SpatialPoint>, PointCloudBounds), Box<dyn std::error::Error>> {
        let file = File::open(&candidate.path)?;

        let buf_reader = BufReader::new(file);
        let mut reader = Reader::new(buf_reader)?;

        // --- First pass: collect raw coordinates & compute bounds ---
        let mut raw_points = Vec::new();
        let asset_bound = calculate_bounds(&candidate.path).unwrap();

        for point_result in reader.points() {
            let point = point_result?;
            let (x, y, z) = transform_coordinates(point.x, point.y, point.z);
            raw_points.push((point, (x, y, z)));
        }

        // Center all axes including Y (not ground-aligned)
        let center_x = (asset_bound.min_x + asset_bound.max_x) * 0.5;
        let center_y = (asset_bound.min_y + asset_bound.max_y) * 0.5; // Center Y, not min
        let center_z = (asset_bound.min_z + asset_bound.max_z) * 0.5;

        // --- Second pass: build spatial points using adjusted positions ---
        let mut asset_points = Vec::with_capacity(raw_points.len());

        for (point, (x, y, z)) in raw_points {
            let centered_x = x - center_x;
            let centered_y = y - center_y;
            let centered_z = z - center_z;
            let classification = u8::from(point.classification);
            let color = point.color.map(|c| (c.red, c.green, c.blue));

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

            asset_points.push(SpatialPoint {
                world_pos: (centered_x, centered_y, centered_z),
                norm_pos: (
                    asset_bound.normalize_x(x),
                    asset_bound.normalize_y(y),
                    asset_bound.normalize_z(z),
                ),
                morton_index: 0,
                spatial_cell_id: 0,
                classification,
                color,
                object_number,
            });
        }

        println!(
            "Loaded {} points from {}",
            asset_points.len(),
            candidate.name
        );

        Ok((asset_points, asset_bound))
    }
}
