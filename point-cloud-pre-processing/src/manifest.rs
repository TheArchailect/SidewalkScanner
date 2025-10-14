/// Scene manifest generation for unified terrain and asset integration.
use crate::atlas::AssetAtlasInfo;
use crate::bounds::PointCloudBounds;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct ClassType {
    pub class_name: String,
    pub objects_ids: HashSet<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct ClassificationInfo {
    pub class_types: HashMap<u8, ClassType>,
}

impl ClassificationInfo {
    pub fn insert_or_update(&mut self, class_id: u8, name: String, object_id: u32) {
        self.class_types
            .entry(class_id)
            .or_insert_with(|| ClassType {
                class_name: name,
                objects_ids: HashSet::new(),
            })
            .objects_ids
            .insert(object_id);
    }
}

impl std::fmt::Debug for ClassificationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassificationInfo")
            .field("class_types", &self.class_types)
            .finish()
    }
}

/// Unified scene manifest linking terrain and asset atlas data.
/// Contains all necessary information for GPU renderer integration.
#[derive(Serialize, Deserialize)]
pub struct SceneManifest {
    /// Primary terrain dataset information and file paths.
    pub terrain: TerrainInfo,
    /// Optional asset atlas configuration and metadata.
    pub asset_atlas: Option<AssetAtlasInfo>,
    // /// Global scene bounds encompassing terrain and all assets.
    // pub scene_bounds: PointCloudBounds,
    /// Describes the class types and object id's found in the specific dataset
    pub classes: ClassificationInfo,
}

/// Terrain dataset information for main point cloud processing.
/// Tracks spatial bounds, texture files, and processing statistics.
#[derive(Serialize, Deserialize)]
pub struct TerrainInfo {
    /// Terrain texture filenames organized by type and format.
    pub texture_files: TerrainTextureFiles,
    /// Spatial coordinate bounds of the terrain dataset.
    pub bounds: PointCloudBounds,
    pub point_count: usize,
    pub has_colour: bool,
}

/// Terrain texture file paths with organized directory structure.
/// References textures in terrain subdirectory for clean organization.
#[derive(Serialize, Deserialize)]
pub struct TerrainTextureFiles {
    /// Position texture (RGBA32F) containing XYZ coordinates.
    pub position: String,
    /// Color and classification texture (RGBA32F) with RGB+class data.
    pub colour_class: String,
    /// Spatial index texture (RGBA32F) with Morton codes for traversal.
    pub spatial_index: String,
    /// Height field texture (R32F) for terrain surface reconstruction.
    pub heightmap: String,
}

/// Scene manifest generator for unified terrain and asset output.
/// Handles manifest creation, validation, and file organization.
pub struct ManifestGenerator {
    /// Base output directory for all generated files.
    output_dir: std::path::PathBuf,
    /// Programmatic name for consistent file identification.
    output_name: String,
}

impl ManifestGenerator {
    pub fn new(output_dir: &Path, output_name: &str) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
            output_name: output_name.to_string(),
        }
    }

    /// Generates unified scene manifest combining terrain and asset data.
    /// Creates final manifest.json file linking all processed resources.
    pub fn generate_unified_manifest(
        &self,
        terrain_info: TerrainInfo,
        asset_atlas_info: Option<AssetAtlasInfo>,
        classes: ClassificationInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate global scene bounds encompassing terrain and assets.
        // let scene_bounds = self.calculate_global_bounds(&terrain_info, &asset_atlas_info);

        let manifest = SceneManifest {
            terrain: terrain_info,
            asset_atlas: asset_atlas_info,
            // scene_bounds,
            classes,
        };

        // Write manifest to root output directory for easy discovery.
        let manifest_path = self.output_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json)?;

        println!("Generated unified manifest: {}", manifest_path.display());
        self.print_manifest_summary(&manifest);

        Ok(())
    }

    /// Prints manifest summary for verification and debugging.
    /// Displays key statistics about processed terrain and assets.
    fn print_manifest_summary(&self, manifest: &SceneManifest) {
        println!("Manifest Summary:");
        println!("  Terrain points: {}", manifest.terrain.point_count);
        println!(
            "  Terrain bounds: ({:.2}, {:.2}) to ({:.2}, {:.2})",
            manifest.terrain.bounds.min_x,
            manifest.terrain.bounds.min_z,
            manifest.terrain.bounds.max_x,
            manifest.terrain.bounds.max_z
        );

        if let Some(atlas) = &manifest.asset_atlas {
            println!(
                "  Asset atlas: {} assets in {}x{} texture",
                atlas.assets.len(),
                atlas.atlas_config.atlas_size,
                atlas.atlas_config.atlas_size
            );

            // Calculate total asset points for statistics.
            let total_asset_points: u32 = atlas.assets.iter().map(|a| a.point_count).sum();
            println!("  Asset points: {}", total_asset_points);
        } else {
            println!("  No asset atlas generated");
        }
    }
}
