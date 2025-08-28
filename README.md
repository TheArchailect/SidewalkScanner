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

## Build Process

### 1. Build Post-Build Script

```bash
# From project root
rustc ./point-cloud-render-engine/scripts/html_config.rs -o ./point-cloud-render-engine/scripts/html_config.bin
```

### 2. Build Web Application

```bash
# Generate WASM and web assets (from project root)
trunk build --release
```

**Trunk Configuration** (`Trunk.toml`):

```toml
[build]
target = "./point-cloud-render-engine/index.html"
html_output = "SidewalkScanner.html"
dist = "./frontend/public/renderer"
filehash = false
no_sri = true

[[copy]]
source = "assets"
target = "assets"

[[hooks]]
stage = "post_build"
command = "./point-cloud-render-engine/scripts/html_config.bin"
```

### 3. Start Frontend Server

```bash
# Install dependencies (first time only)
npm install

# Start development server
npm run dev
```

## Usage

### Process Point Cloud Data

```bash
# Convert LAZ/LAS to unified texture format
cargo run --bin point-cloud-pre-processing <path to your input file>.laz
```

This generates:

- `input_file_position_2048x2048.dds` - RGBA32F: XYZ coordinates + validity
- `input_file_colour_class_2048x2048.dds` - RGBA32F: RGB colour + classification
- `input_file_spatial_index_2048x2048.dds` - RGBA32F: Morton codes + spatial data
- `input_file_heightmap_2048x2048.dds` - R32F: Road surface elevation
- `input_file_metadata_2048x2048.json` - Bounds and processing statistics

### Configure Asset Path

Update `RELATIVE_ASSET_PATH` in `main.rs`:

```rust
const RELATIVE_ASSET_PATH: &'static str = "pre_processor_data/your_file_name";
```

### Run Application

```bash
# Native desktop
cargo run --bin point-cloud-render-engine

# Web (after trunk build)
npm run dev
# Navigate to http://localhost:<your_port>
```

## Development Controls (no integration with the front end in the current release)

### Camera Navigation

- **Mouse Wheel**: Zoom in/out
- **Middle Mouse + Drag**: Pan view
- **Space**: Follow mouse cursor on terrain
- **A/D**: Rotate around focus point

### Render Modes

- **Z**: RGB Colour (default)
- **X**: Original Classification
- **C**: Modified Classification
- **V**: Morton Code Debug
- **B**: Performance Debug
- **N**: Class Selection Mode

### Polygon Classification Tool

- **P**: Toggle polygon tool
- **1-9**: Set classification class
- **Left Click**: Add polygon vertex
- **Left Shift**: Complete polygon
- **O**: Clear current polygon
- **I**: Clear all polygons

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
- **Web**: Chrome, Firefox, Safari (WebGPU required)

## Minimum Requirements

- **GPU**: WebGPU-compatible (DirectX 12, Vulkan, or Metal)
- **RAM**: 6GB recommended for large datasets
- **Browser**: Chrome 113+, Firefox 121+, Safari 17+ (for web deployment)
