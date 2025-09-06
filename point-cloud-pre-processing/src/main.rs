mod asset_processor;
mod atlas;
mod bounds;
mod constants;
mod converter;
mod dds_writer;
mod heightmap;
mod manifest;
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
            converter.convert_with_assets()?;
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
