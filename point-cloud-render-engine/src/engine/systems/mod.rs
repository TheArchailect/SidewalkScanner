//! Core runtime systems for rendering control and diagnostics.
//!
//! Provides render mode switching, FPS tracking, and pipeline debugging
//! utilities for development and user interaction.

/// Pipeline debugging utilities for render phase verification.
///
/// Keyboard-triggered diagnostic output showing entity counts, phase items, and pipeline state.
pub mod debug_pipeline;

/// FPS tracking and notification systems for performance monitoring.
///
/// Sends frame rate updates to frontend via RPC and updates native UI overlays.
pub mod fps_tracking;

/// Render mode state management and switching system.
///
/// Handles keyboard input (native) or RPC notifications (WASM) for classification view modes.
pub mod render_mode;
