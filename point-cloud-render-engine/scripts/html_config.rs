use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    //  Environment variable parameter - the distributed directory
    let staging_dir = env::var("TRUNK_STAGING_DIR").unwrap();
    
    //  Using the configured html output name (set in Trunk.toml)
    let staged_html_path = PathBuf::from(staging_dir).join("SidewalkScanner.html");

    // Read the html file
    let html_change = fs::read_to_string(&staged_html_path).unwrap();

    // Implement the change
    let html_change = html_change
        .replace("/point-cloud-render-engine.js", "./point-cloud-render-engine.js")
        .replace("/point-cloud-render-engine_bg.wasm", "./point-cloud-render-engine_bg.wasm");

    // Write
    fs::write(staged_html_path, html_change).unwrap();
}