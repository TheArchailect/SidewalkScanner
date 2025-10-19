use las::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Create LAS file reader for point cloud access.
/// Handles both .las and .laz compressed formats.
pub fn create_reader(file_path: &Path) -> Result<Reader, Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let buf_reader = BufReader::new(file);
    Ok(Reader::new(buf_reader)?)
}
