// build.rs
use std::{env, fs, path::PathBuf};


fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let test_data = serde_json::json!({
        "name": "Test User",
        "age": 30,
        "is_active": true,
        "name:": "Jo"
    });

    // Also write to a location that can be copied to dist
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let assets_dir = manifest_dir.join("assets");
    fs::create_dir_all(&assets_dir).ok(); // Create assets dir if it doesn't exist

    let json_content = serde_json::to_string_pretty(&test_data).unwrap();

    // Write to OUT_DIR (for include_str! in Rust code)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let json_path = out_dir.join("test_data.json");
    fs::write(&json_path, &json_content).expect("Failed to write test_data.json to OUT_DIR");

    

    let public_json_path = assets_dir.join("test_data.json");
    fs::write(&public_json_path, &json_content).expect("Failed to write test_data.json to assets");

    println!("cargo:warning=Generated JSON in assets/test_data.json");
}

/*
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let test_data = serde_json::json!({
        "name": "Test User",
        "age": 30,
        "is_active": true,
        "name:": "Jo"
    });

    let json_content = serde_json::to_string_pretty(&test_data).unwrap();

    // Write to OUT_DIR (for include_str! in Rust code)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let json_path = out_dir.join("test_data.json");
    fs::write(&json_path, &json_content).expect("Failed to write test_data.json to OUT_DIR");

    // Also write to a location that can be copied to dist
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let assets_dir = manifest_dir.join("assets");
    fs::create_dir_all(&assets_dir).ok(); // Create assets dir if it doesn't exist

    let public_json_path = assets_dir.join("test_data.json");
    fs::write(&public_json_path, &json_content).expect("Failed to write test_data.json to assets");

    println!("cargo:warning=Generated JSON in assets/test_data.json");
}*/