# S-CIT

High-performance web-based 3D point cloud editor built with Bevy and Rust.

## Features

- GPU-accelerated rendering of millions of points
- Unified RGBA32F texture pipeline for positions, colours, and classifications
- Real-time polygon classification with compute shaders
- Eye Distance Lighting (EDL) depth enhancement
- Spatial indexing with Z-order curve optimisation
- Cross-platform: native desktop and WebGPU web deployment

## Prerequisites

### Core Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install trunk for web builds
cargo install trunk
```

### Frontend Dependencies

```bash
# Install Node.js 20+
# Download from https://nodejs.org/

# Install frontend dependencies
npm install
```

## Pre-Build Generate Assets

### Process Point Cloud Data

```bash
# Convert LAZ/LAS to unified texture format
cargo run --bin point-cloud-pre-processing <path to your input laz file>.laz <asset_dir> <output dir>
# For example the default expected asset location will be:
cargo run --bin point-cloud-pre-processing
/home/repository/point-cloud-render-engine/assets/riga_numbered_0.05.laz
/home/repository/point-cloud-render-engine/assets/placeable_assets
/home/repository/point-cloud-render-engine/assets/output
```

This generates:

- `input_file_position_2048x2048.dds` - RGBA32F: XYZ coordinates + connectivitcay class id (a unique id for instances of a classification)
- `input_file_colour_class_2048x2048.dds` - RGBA32F: RGB colour + classification
- `input_file_spatial_index_2048x2048.dds` - RGBA32F: Morton codes + spatial data
- `input_file_heightmap_2048x2048.dds` - R32F: Road surface elevation
- `input_file_metadata_2048x2048.json` - Bounds and processing statistics

## Build Process

### 1. Build Post-Build Script

note: this script only needs to be compiled for your system once.

```bash
# From project root
rustc ./point-cloud-render-engine/scripts/html_config.rs -o ./point-cloud-render-engine/scripts/html_config.bin
```

### 2. Build Web Application

```bash
# Generate WASM and web assets (from project root)
trunk build --release
```

### 3. Start Frontend Server

```bash
# if you use NVM
nvm use 20

# Install dependencies (first time only)
npm install

# Start development server
npm run dev
```

### Configuration details in constants crate

```rust
/// Generated Asset Details
pub const RELATIVE_MANIFEST_PATH: &'static str = "output/";
pub const TERRAIN_PATH: &'static str = "/terrain/";
pub const ASSET_PATH: &'static str = "/assets/AssetAtlas/";
pub const TEXTURE_RESOLUTION_FILE_PATH: &'static str = "2048x2048";

/// Road classification codes for heightmap generation
pub const ROAD_CLASSIFICATIONS: &[u8] = &[2, 10, 11, 12];

/// Coordinate transform for input LiDAR PC Coordinate System
pub const COORDINATE_TRANSFORM: [[f64; 3]; 3] = [
    [1.0, 0.0, 0.0],  // X = X
    [0.0, 0.0, 1.0],  // Y = Z
    [0.0, -1.0, 0.0], // Z = -Y
];

/// The maximum number of points the compute shader expects to perform re-classification and hide operations
pub const MAXIMUM_POLYGON_POINTS: usize = 2048;

/// The maximum number of polygons the compute shader expects to perform re-classification and hide operations
pub const MAXIMUM_POLYGONS: usize = 512;

/// The maximum number of polygons the compute shader expects to perform re-classification and hide operations
pub const MAX_IGNORE_MASK_LENGTH: usize = 512;


/// EDL lighting config
pub const DRAW_LINE_WIDTH: f32 = 0.076;
pub const MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE: f32 = 0.125;
pub const DRAW_VERTEX_SIZE: f32 = 0.08;

/// Texture DDS settings:
/// Unified texture resolution for all generated textures (positions, colour+class, heightmap)
pub const TEXTURE_SIZE: usize = 2048;

/// Maximum points that can fit in a texture
pub const MAX_POINTS: usize = TEXTURE_SIZE * TEXTURE_SIZE;

/// Heightmap blend radius for road surface smoothing (pixels)
pub const HEIGHTMAP_BLEND_RADIUS: f32 = 64.0;

/// Sample size for colour detection
pub const COLOUR_DETECTION_SAMPLE_SIZE: usize = 100;
```

### Run Application

```bash
# Native desktop (Development environment only)
cargo run --bin point-cloud-render-engine

# Web (after trunk build)
npm run dev
# Navigate to http://localhost:<your_port>
```

## Development Controls (no integration with the front end in the current release)

### Camera Navigation

- **Mouse Wheel**: Zoom in/out
- **Right Mouse + Drag**: Free look
- **WASD**: Move around
- **Q/E**: Up and down
- **R/F** Pitch camera
- **Hold + Shift**: Move faster
- **Hold + Ctrl**: Move slower

### Render Modes

- **Z**: RGB Colour (default)
- **X**: Original Classification
- **C**: Modified Classification
- **V**: Morton Code Debug
- **B**: Performance Debug
- **N**: Class Selection Mode

## Technical Specifications

### Texture Pipeline

- **Resolution**: 2048×2048 (configurable via `TEXTURE_SIZE` constant)
- **Format**: 32-bit float textures for precision
- **Capacity**: maximum number of points is determined by `TEXTURE_SIZE`: 1k -> 8k (~1 million -> ~67 million) points
- **Coordinate Transform**: -90° X rotation (Z→Y, -Y→Z, X→X)

### Performance Features

- Spatial indexing using Z-order curves
- GPU compute shaders for classification modification tools
- Parallel processing with Rayon
- Eye Distance Lighting for depth perception

### File Support

- **Input**: LAZ/LAS files with optional RGB data
- **Output**: DDS textures + JSON metadata

## Platform Support

- **Desktop**: Windows, macOS, Linux (native performance)
- **Web**: Chrome, Firefox, Safari (Experimental WebGPU required)

## Minimum Requirements

- **GPU**: WebGPU-compatible (DirectX 12, Vulkan, or Metal)
- **RAM**: 6GB recommended for large datasets
- **Browser**: Chrome 113+, Firefox 121+, Safari 17+ (for web deployment)
