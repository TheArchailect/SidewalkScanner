//! Core engine systems for point cloud rendering and scene management.
//!
//! Implements the complete rendering pipeline from asset loading through GPU
//! compute shaders to final frame output, with custom render phases for
//! point cloud visualisation and instanced asset rendering.
//!
//! ## Architecture Overview
//!
//! The engine is organised into distinct subsystems that work together to
//! process and render large-scale point cloud datasets:
//!
//! ### Data Flow Pipeline
//!
//! ```text
//! Manifest Loading → Texture Loading → Configuration → Point Cloud Creation
//!        ↓                                                      ↓
//!   Scene Bounds                                    GPU Vertex Buffers
//!        ↓                                                      ↓
//!   Camera Setup                                    Custom Render Phase
//!        ↓                                                      ↓
//!   Extract to Render World ←──────────────────────────────────┘
//!        ↓
//!   Compute Shaders (Classification + EDL Depth)
//!        ↓
//!   Point Cloud Rendering + Instanced Assets
//!        ↓
//!   Post-Processing (EDL Enhancement)
//!        ↓
//!   Final Frame Output
//! ```
//!
//! ## Module Responsibilities
//!
//! ### Asset Management (`assets`)
//! Loads and manages scene manifests, texture references, spatial bounds,
//! and asset definitions. Provides unified access to terrain and atlas data
//! for rendering systems.
//!
//! ### Camera Control (`camera`)
//! Viewport camera with orbit controls, heightmap-aware ground plane intersection,
//! and smooth interpolation for navigation.
//!
//! ### GPU Compute (`compute`)
//! Non-destructive point classification via polygon masks and spatial filtering.
//! Generates depth buffers for eye-dome lighting shading enhancement.
//!
//! ### Application Core (`core`)
//! Application lifecycle management with state machine transitions from loading
//! through asset configuration to runtime execution.
//!
//! ### Asset Loading (`loading`)
//! Multi-stage loading pipeline with progress tracking: manifest parsing, texture
//! loading, format configuration, and point cloud entity creation.
//!
//! ### Mesh Generation (`mesh`)
//! Indexed vertex buffer generation for GPU-side point expansion into
//! screen-aligned quads via vertex shaders.
//!
//! ### Rendering (`render`)
//! Custom render pipelines for point clouds and instanced assets with specialised
//! material bindings, EDL post-processing, and resource extraction systems.
//!
//! ### Scene Utilities (`scene`)
//! Heightmap sampling, heightfield-aware grid generation, and interactive
//! gizmos for viewport feedback.
//!
//! ### Runtime Systems (`systems`)
//! Render mode switching, FPS tracking, and pipeline debugging utilities
//! for development and performance monitoring.
//!
//! ## Render World Extraction
//!
//! The engine uses Bevy's render world separation pattern:
//! - Main world handles gameplay logic and user interaction
//! - Render world receives extracted resources each frame
//! - Extraction systems in `render::extraction` synchronise state
//!
//! Critical extracted resources:
//! - `PointCloudRenderState`: Camera position and scene bounds for uniforms
//! - `SceneManifest`: Terrain bounds and asset atlas metadata
//! - `RenderModeState`: Classification view mode for compute shaders
//! - `PolygonClassificationData`: Active polygon masks for compute pipeline
//!
//! ## Custom Render Phases
//!
//! ### PointCloudPhase
//! Depth-sorted rendering of terrain point clouds with GPU vertex expansion.
//! Integrates with Bevy's depth buffer for proper occlusion with standard meshes.
//!
//! ### Transparent3d (Instanced Assets)
//! Standard Bevy phase extended with per-instance vertex buffers for GPU-efficient
//! asset placement with atlas texture sampling.
//!
//! ## Compute Pipeline Integration
//!
//! Compute shaders execute during `RenderSet::Queue` before render phases:
//!
//! 1. **Classification Compute**: Applies polygon masks to reclassify/hide points
//! 2. **Depth Compute**: Generates camera-space depth from world positions
//! 3. **Point Cloud Render**: Uses computed classification and depth textures
//! 4. **EDL Post-Process**: Enhances depth perception via neighbouring pixel comparison

/// Asset loading and management for scene manifests and texture references.
pub mod assets;

/// Viewport camera with orbit controls and heightmap-aware intersection.
pub mod camera;

/// GPU compute shaders for point classification and depth buffer generation.
pub mod compute;

/// Application lifecycle and state machine for loading progression.
pub mod core;

/// Multi-stage asset loading pipeline with progress tracking.
pub mod loading;

/// Mesh generation for GPU-side vertex expansion into screen-aligned quads.
pub mod mesh;

/// Custom render pipelines for point clouds, instanced assets, and post-processing.
pub mod render;

/// Scene utilities for heightmap sampling, grid generation, and gizmos.
pub mod scene;

/// Runtime systems for render mode switching and performance monitoring.
pub mod systems;
