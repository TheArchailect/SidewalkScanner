//! Point cloud preprocessing pipeline for terrain and asset library processing.
//!
//! Converts LAS/LAZ point clouds into GPU-optimised texture atlases with spatial
//! organisation, heightmap generation, and unified manifest output.

/// Asset library processing workflow and atlas generation orchestration.
mod asset_processor;

/// Asset atlas data structures, texture generation, and tile management.
mod atlas;

/// Point cloud coordinate bounds calculation and normalisation.
mod bounds;

/// Main point cloud converter orchestrating terrain and asset processing pipelines.
mod converter;

/// DDS texture file writer with unified 32-bit float formats.
mod dds_writer;

/// Fast parallel heightmap generation with flood-fill and Gaussian smoothing.
mod heightmap;

/// LAS/LAZ file reader creation for point cloud access.
mod laz;

/// Scene manifest generation linking terrain and asset atlas data.
mod manifest;

/// Z-order spatial layout and texture generation for point cloud data.
mod spatial_layout;

use converter::PointCloudConverter;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Support both old single-file format and new asset library format
    match args.len() {
        2 => {
            // Legacy single file processing
            let input_path = &args[1];
            let output_stem = input_path.trim_end_matches(".laz").trim_end_matches(".las");

            let mut converter = PointCloudConverter::new(input_path, output_stem)?;
            converter.convert()?;
        }
        4 => {
            // New asset library processing: <main_cloud.laz> <asset_library_dir> <output_dir>
            let main_cloud = std::path::Path::new(&args[1]);
            let asset_library_dir = std::path::Path::new(&args[2]);
            let output_dir = std::path::Path::new(&args[3]);

            if !main_cloud.exists() {
                eprintln!(
                    "Error: Main cloud file '{}' does not exist",
                    main_cloud.display()
                );
                std::process::exit(1);
            }

            if !asset_library_dir.is_dir() {
                eprintln!(
                    "Error: Asset library directory '{}' does not exist",
                    asset_library_dir.display()
                );
                std::process::exit(1);
            }

            let mut converter =
                PointCloudConverter::with_asset_library(main_cloud, asset_library_dir, output_dir)?;
            converter.convert()?;
        }
        _ => {
            eprintln!("Usage:");
            eprintln!(
                "  {} <input.laz>                              (single file)",
                args[0]
            );
            eprintln!(
                "  {} <main_cloud.laz> <asset_dir> <output_dir> (with assets)",
                args[0]
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
