//! Asset loading and initialisation systems for point cloud data.
//!
//! Manages the multi-stage loading pipeline from manifest parsing through
//! texture configuration to final scene setup with progress tracking.

/// Scene manifest loading and bounds extraction from JSON configuration.
///
/// Initiates texture loading after manifest parsing and camera initialisation.
pub mod manifest_loader;

/// Point cloud entity creation and scene element spawning.
///
/// Creates GPU-side vertex buffers, ground grid, and visual gizmos after texture loading.
pub mod point_cloud_creator;

/// Loading progress tracking resource for state transitions.
///
/// Monitors completion of bounds, textures, configuration, and compute pipeline setup.
pub mod progress;

/// Texture format configuration for compute pipeline compatibility.
///
/// Configures sampling modes and creates storage-capable textures for shader access.
pub mod texture_config;

/// DDS texture loading state monitoring for terrain and asset data.
///
/// Tracks load completion of position, colour, heightmap, and spatial index textures.
pub mod texture_loader;
