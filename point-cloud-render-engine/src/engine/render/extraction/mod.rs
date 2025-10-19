//! Resource extraction systems for render world synchronisation.
//!
//! Transfers main world state to render world each frame during Bevy's extract schedule,
//! ensuring GPU pipelines have current camera data, bounds, and application state.

/// Application state extraction for render world access.
///
/// Transfers AppState to render world for state-conditional rendering systems.
pub mod app_state;

/// Camera render phase management and entity retention.
///
/// Maintains point cloud render phases per active camera with entity lifecycle tracking.
pub mod camera_phases;

/// Point cloud render state extraction with camera and bounds data.
///
/// Captures camera position and scene bounds for GPU uniform buffer generation.
pub mod render_state;

/// Scene manifest extraction for render world texture access.
///
/// Transfers manifest resource containing terrain bounds and asset atlas metadata.
pub mod scene_manifest;
