//! Scene visualisation and terrain interaction utilities.
//!
//! Provides heightmap sampling, heightfield-aware grid generation,
//! and interactive gizmos for cursor feedback and direction indication.

/// Direction and mouse intersection gizmos for viewport interaction feedback.
///
/// Displays cursor position on terrain and movement direction indicators with tool-aware visibility.
pub mod gizmos;

/// Heightfield-aware grid mesh generation following terrain elevation.
///
/// Creates adaptive grid lines that conform to heightmap data for spatial reference.
pub mod grid;

/// Heightmap sampling utilities for terrain intersection queries.
///
/// Bilinear interpolation of heightmap textures for smooth elevation sampling.
pub mod heightmap;
