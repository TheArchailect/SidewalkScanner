# Point Cloud Render Engine

A high-performance web-based 3D point cloud editor built with Bevy and Rust.

## Prerequisites

```bash
# Install Rust
# Windows Installer:
https://rustup.rs

# Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target for web builds
rustup target add wasm32-unknown-unknown

# Install trunk for WASM builds
cargo install trunk
```

## Usage

### 1. Process Point Cloud Data

Convert LAZ files to GPU-optimized textures:

```bash
cargo run --bin point-cloud-pre-processing /path/to/your/file.laz
```

This generates DDS textures and metadata in `/point-cloud-render-engine/assets/`

### 2. Development (Native)

```bash
cargo run
```

### 3. Web Deployment

```bash
# Build WASM
trunk build --release

# Serve locally
python3 serve.py
```

## Controls

_Controls documentation coming soon..._

## Requirements

- WebGPU supported browser
