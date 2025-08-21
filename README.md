# S-CIT

A high-performance web-based 3D point cloud editor built with Bevy and Rust.

## Features

- GPU-accelerated rendering of millions of points
- Unified RGBA32F texture pipeline for positions, colours, and classifications
- Real RGB colour support from LAZ files
- Interactive polygon classification tool

## Prerequisites

```bash
# Install Rust
https://rustup.rs

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk for web builds
cargo install trunk
```

## Texture Pipeline

### Generated Textures (1024x1024 | 2048×2048 | 4090x4090)

1. **Position** (`_position_resolutionxresolution.dds`)
   - RGBA32F: XYZ coordinates (normalised) + validity

2. **Colour+Classification** (`_colour_class_resolutionxresolution.dds`)
   - RGBA32F: RGB colour + classification value

3. **Heightmap** (`_heightmap_resolutionxresolution.dds`)
   - R32F: Road surface elevation (normalised)

4. **Metadata** (`_metadata_resolutionxresolution.json`)
   - Coordinate bounds and processing statistics

## Usage

### 1. Process LAZ Files

```bash
cargo run --bin point-cloud-pre-processing /path/to/file.laz
```

### 2. Configure Asset Path

Update `main.rs`:

```rust
const RELATIVE_ASSET_PATH: &'static str = "encoded_textures/your_file";
```

### 3. Run

```bash
# Native
cargo run --bin point-cloud-render-engine

# Web
trunk build --release
python3 serve.py
```

## Controls

### Camera & Navigation

- **Mouse Wheel**: Zoom
- **Middle Mouse**: Drag/Pan
- **Space**: Follow towards mouse ray-cast
- **A/D**: Orient around focus point

### Render Modes

- **Z**: RGB Colour
- **X**: Original Classification
- **C**: Modified Classification
- **V**: Morton Code
- **B**: Performance Debug
- **N**: Class Selection

### Polygon Classification Tool

- **P**: Toggle polygon tool on/off
- **1-9**: Set current classification class (1-9)
- **Left Click**: Add polygon point
- **Left Shift**: Complete current polygon
- **O**: Clear current polygon being drawn
- **I**: Clear all completed polygons

## Technical Specifications

### File Support

- Input: LAZ/LAS with optional RGB colour
- Output: DDS textures (32-bit float) + JSON metadata

## Minimum Requirements

- WebGPU-compatible browser
- Integrated graphics
