//! Core application setup and state management.
//!
//! Handles application lifecycle, window configuration, state transitions,
//! and plugin initialisation for both native and WASM targets.

/// Application setup and plugin configuration for the Bevy engine.
///
/// Creates the main app with rendering pipelines, asset loading systems,
/// and platform-specific configurations.
pub mod app_setup;

/// Application state machine and loading progress transitions.
///
/// Manages states from initial loading through asset configuration to runtime execution.
pub mod app_state;

/// Platform-specific window configuration for native and WASM builds.
///
/// Configures canvas integration for web targets and vsync settings.
pub mod window_config;
