/// Point cloud to texture converter main entry point
mod bounds;
mod constants;
mod converter;
mod dds_writer;
mod heightmap;
mod spatial_layout;

use converter::PointCloudConverter;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input.laz>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_stem = input_path.trim_end_matches(".laz").trim_end_matches(".las");

    let mut converter = PointCloudConverter::new(input_path, output_stem)?;
    converter.convert()?;

    Ok(())
}
