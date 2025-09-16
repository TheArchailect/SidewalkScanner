/// Asset library atlas generation for efficient GPU rendering
use crate::bounds::PointCloudBounds;
use crate::constants::TEXTURE_SIZE;
use crate::dds_writer::write_f32_texture;
use crate::spatial_layout::SpatialPoint;
use indicatif::{ProgressBar, ProgressStyle};
use las::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Error types for atlas processing operations.
#[derive(Debug)]
pub enum AtlasError {
    IoError(std::io::Error),
    LasError(las::Error),
    TileOverflow,
    InvalidAssetData,
}

impl From<std::io::Error> for AtlasError {
    fn from(err: std::io::Error) -> Self {
        AtlasError::IoError(err)
    }
}

impl From<las::Error> for AtlasError {
    fn from(err: las::Error) -> Self {
        AtlasError::LasError(err)
    }
}

impl std::fmt::Display for AtlasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtlasError::IoError(e) => write!(f, "IO error: {}", e),
            AtlasError::LasError(e) => write!(f, "LAS error: {}", e),
            AtlasError::TileOverflow => write!(f, "Too many assets for atlas tiles"),
            AtlasError::InvalidAssetData => write!(f, "Invalid asset point cloud data"),
        }
    }
}

impl std::error::Error for AtlasError {}

/// UV coordinate bounds for atlas tile access in normalized space.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AtlasRegion {
    /// Minimum UV coordinates (top-left corner).
    pub uv_min: [f32; 2],
    /// Maximum UV coordinates (bottom-right corner).
    pub uv_max: [f32; 2],
}

impl AtlasRegion {
    /// Calculates UV bounds from atlas tile position and dimensions.
    /// Grid coordinates are converted to normalized UV space for GPU sampling.
    pub fn from_tile_position(tile_pos: (u32, u32), tile_size: u32, atlas_size: u32) -> Self {
        let tiles_per_axis = atlas_size / tile_size;
        let u_step = 1.0 / tiles_per_axis as f32;
        let v_step = 1.0 / tiles_per_axis as f32;

        let u_min = tile_pos.0 as f32 * u_step;
        let v_min = tile_pos.1 as f32 * v_step;
        let u_max = u_min + u_step;
        let v_max = v_min + v_step;

        Self {
            uv_min: [u_min, v_min],
            uv_max: [u_max, v_max],
        }
    }
}

/// Metadata for individual assets within the atlas system.
/// Contains both spatial and texture coordinate information for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Original filename without extension for identification.
    pub name: String,
    /// Atlas tile coordinates in grid space (not UV).
    pub atlas_position: (u32, u32),
    /// UV bounds within the atlas texture for GPU sampling.
    pub uv_bounds: AtlasRegion,
    /// Asset-local coordinate bounds for spatial queries.
    pub local_bounds: PointCloudBounds,
    /// Point count loaded into this atlas tile.
    pub point_count: u32,
}

/// Configuration parameters for atlas generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConfig {
    /// Total atlas texture resolution (typically 2048x2048).
    pub atlas_size: u32,
    /// Individual tile resolution within atlas (typically 256x256).
    pub tile_size: u32,
    /// Maximum assets that can fit (calculated from dimensions).
    pub max_assets: u32,
}

impl AtlasConfig {
    /// Creates atlas configuration with standard tile layout.
    /// Uses 256x256 tiles within a 2048x2048 atlas for 64 total assets.
    pub fn standard() -> Self {
        let atlas_size = TEXTURE_SIZE as u32;
        let tile_size = 256u32;
        let tiles_per_axis = atlas_size / tile_size;
        let max_assets = tiles_per_axis * tiles_per_axis;

        Self {
            atlas_size,
            tile_size,
            max_assets,
        }
    }
}

/// Texture file names for the generated atlas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasTextureFiles {
    /// Position texture filename (RGBA32F).
    pub position: String,
    /// Color and classification texture filename (RGBA32F).
    pub colour_class: String,
}

/// Asset atlas information for renderer integration.
/// Contains metadata and configuration for instanced rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetAtlasInfo {
    /// Atlas texture filenames.
    pub texture_files: AtlasTextureFiles,
    /// Individual asset metadata for instancing.
    pub assets: Vec<AssetMetadata>,
    /// Atlas configuration parameters.
    pub atlas_config: AtlasConfig,
}

/// Complete atlas texture set with raw data arrays.
pub struct AtlasTextureSet {
    /// Position texture data (RGBA32F format).
    pub position_data: Vec<f32>,
    /// Color and classification texture data (RGBA32F format).
    pub colour_class_data: Vec<f32>,
}

/// Asset candidate discovered during library scanning.
/// Contains basic validation information before processing.
#[derive(Debug)]
pub struct AssetCandidate {
    /// Full path to the asset file.
    pub path: PathBuf,
    /// Asset name derived from filename.
    pub name: String,
    /// Point count from LAS header.
    pub point_count: u64,
}

/// Atlas-aware texture generator for multiple point cloud assets.
/// Manages spatial organization and texture generation for GPU rendering.
pub struct AtlasTextureGenerator {
    /// Atlas configuration parameters.
    config: AtlasConfig,
    /// Asset data organized by atlas tile position for spatial locality.
    asset_tiles: HashMap<(u32, u32), Vec<SpatialPoint>>,
    /// Metadata for each processed asset.
    asset_metadata: Vec<AssetMetadata>,
    /// Current tile assignment counter.
    next_tile_position: (u32, u32),
}

impl AtlasTextureGenerator {
    /// Creates new atlas generator with standard configuration.
    /// Initializes empty tile grid and metadata tracking.
    pub fn new() -> Self {
        let config = AtlasConfig::standard();

        Self {
            config,
            asset_tiles: HashMap::new(),
            asset_metadata: Vec::new(),
            next_tile_position: (0, 0),
        }
    }

    /// Adds point cloud asset to specified atlas tile position.
    /// Normalizes coordinates to tile-local space for efficient packing.
    pub fn add_asset(
        &mut self,
        asset_data: &[SpatialPoint],
        asset_name: String,
        local_bounds: PointCloudBounds,
    ) -> Result<(), AtlasError> {
        // Check if we have space for another asset.
        let tiles_per_axis = self.config.atlas_size / self.config.tile_size;
        if self.next_tile_position.1 >= tiles_per_axis {
            return Err(AtlasError::TileOverflow);
        }

        let tile_pos = self.next_tile_position;

        // Calculate UV bounds for this tile.
        let uv_bounds = AtlasRegion::from_tile_position(
            tile_pos,
            self.config.tile_size,
            self.config.atlas_size,
        );

        // Store asset data in the tile.
        self.asset_tiles.insert(tile_pos, asset_data.to_vec());

        // Create metadata entry.
        let metadata = AssetMetadata {
            name: asset_name,
            atlas_position: tile_pos,
            uv_bounds,
            local_bounds,
            point_count: asset_data.len() as u32,
        };

        self.asset_metadata.push(metadata);

        // Advance to next tile position.
        self.advance_tile_position();

        Ok(())
    }

    /// Advances to the next available tile position in row-major order.
    /// Handles wrapping to next row when reaching end of current row.
    fn advance_tile_position(&mut self) {
        let tiles_per_axis = self.config.atlas_size / self.config.tile_size;
        self.next_tile_position.0 += 1;

        if self.next_tile_position.0 >= tiles_per_axis {
            self.next_tile_position.0 = 0;
            self.next_tile_position.1 += 1;
        }
    }

    /// Generates position and color/classification atlas textures.
    /// Spatial indexing not required for instanced asset rendering.
    pub fn generate_atlas_textures(&self) -> AtlasTextureSet {
        let atlas_pixels = (self.config.atlas_size * self.config.atlas_size) as usize;
        let mut position_data = vec![0.0f32; atlas_pixels * 4]; // RGBA
        let mut colour_class_data = vec![0.0f32; atlas_pixels * 4]; // RGBA

        // Process each tile.
        for ((tile_x, tile_y), points) in &self.asset_tiles {
            let tile_start_x = tile_x * self.config.tile_size;
            let tile_start_y = tile_y * self.config.tile_size;

            // Fill tile with point data.
            for (point_idx, point) in points.iter().enumerate() {
                if point_idx >= (self.config.tile_size * self.config.tile_size) as usize {
                    break; // Tile full.
                }

                // Calculate pixel position within tile.
                let local_x = point_idx as u32 % self.config.tile_size;
                let local_y = point_idx as u32 / self.config.tile_size;
                let global_x = tile_start_x + local_x;
                let global_y = tile_start_y + local_y;
                let pixel_idx = (global_y * self.config.atlas_size + global_x) as usize * 4;

                if pixel_idx + 3 < position_data.len() {
                    // Position data.
                    position_data[pixel_idx] = point.norm_pos.0;
                    position_data[pixel_idx + 1] = point.norm_pos.1;
                    position_data[pixel_idx + 2] = point.norm_pos.2;
                    position_data[pixel_idx + 3] = point.object_number / 121.0;

                    // Color and classification data.
                    if let Some((r, g, b)) = point.color {
                        colour_class_data[pixel_idx] = r as f32 / 65535.0;
                        colour_class_data[pixel_idx + 1] = g as f32 / 65535.0;
                        colour_class_data[pixel_idx + 2] = b as f32 / 65535.0;
                    } else {
                        colour_class_data[pixel_idx] = 1.0;
                        colour_class_data[pixel_idx + 1] = 1.0;
                        colour_class_data[pixel_idx + 2] = 1.0;
                    }
                    colour_class_data[pixel_idx + 3] = point.classification as f32 / 255.0;
                }
            }
        }

        AtlasTextureSet {
            position_data,
            colour_class_data,
        }
    }

    /// Returns asset metadata for all processed assets.
    /// Used for generating manifest files and renderer integration.
    pub fn get_asset_metadata(&self) -> &[AssetMetadata] {
        &self.asset_metadata
    }

    /// Returns atlas configuration parameters.
    /// Provides access to sizing and layout information.
    pub fn get_config(&self) -> &AtlasConfig {
        &self.config
    }
}

/// Discovers and validates asset library files for atlas generation.
/// Filters files by extension and validates point cloud structure.
pub fn discover_asset_files(asset_dir: &Path) -> Result<Vec<AssetCandidate>, AtlasError> {
    let mut candidates = Vec::new();

    // Scan directory for .laz and .las files.
    for entry in fs::read_dir(asset_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                let ext_str = extension.to_string_lossy().to_lowercase();
                if ext_str == "laz" || ext_str == "las" {
                    // Extract asset name from filename.
                    let name = path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();

                    // Validate by reading header.
                    if let Ok(point_count) = validate_asset_file(&path) {
                        candidates.push(AssetCandidate {
                            path,
                            name,
                            point_count,
                        });
                    }
                }
            }
        }
    }

    candidates.sort_by(|a, b| a.name.cmp(&b.name));

    println!("Asset processing order:");
    for (i, candidate) in candidates.iter().enumerate() {
        println!("  {}: {}", i, candidate.name);
    }

    Ok(candidates)
}

/// Validates an asset file and returns point count.
/// Performs basic header validation without loading all points.
fn validate_asset_file(path: &Path) -> Result<u64, AtlasError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let reader = Reader::new(buf_reader)?;

    let header = reader.header();
    let point_count = header.number_of_points();

    // Basic validation - ensure we have some points.
    if point_count == 0 {
        return Err(AtlasError::InvalidAssetData);
    }

    Ok(point_count)
}

/// Generates programmatic filename from asset name.
/// Converts underscore-separated names to PascalCase format.
pub fn generate_programmatic_name(input: &str) -> String {
    input
        .split('_')
        .map(|part| {
            // Capitalize first letter of each part.
            let mut chars: Vec<char> = part.chars().collect();
            if !chars.is_empty() {
                chars[0] = chars[0].to_uppercase().next().unwrap_or(chars[0]);
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<String>()
}
