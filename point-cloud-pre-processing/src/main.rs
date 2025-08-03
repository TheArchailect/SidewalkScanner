mod bounds;
mod converter;
mod dds_writer;
mod heightmap;
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

    println!("Converting {} to DDS textures (BC6H-ready)...", input_path);

    let mut converter = PointCloudConverter::new(input_path, output_stem)?;
    converter.convert()?;

    println!("Conversion complete!");
    println!("Generated files:");
    println!(
        "  {}_positions.dds - XYZ coordinates (16-bit float RGBA, BC6H-ready)",
        output_stem
    );
    println!(
        "  {}_metadata.dds - Classification, Intensity (8-bit RGBA)",
        output_stem
    );
    println!("  {}_bounds.json - Original coordinate bounds", output_stem);

    Ok(())
}
