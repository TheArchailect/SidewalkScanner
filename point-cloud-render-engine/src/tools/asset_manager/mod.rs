//! Asset placement and manipulation tool for instanced rendering.
//!
//! Provides interactive UI panel (native only) and world-space manipulation
//! for placing, selecting, rotating, and deleting asset instances with
//! real-time heightmap-aware positioning and wireframe visualisation.
//!
//! ## Architecture
//!
//! The asset manager operates in two modes:
//!
//! ### Placement Mode
//! Active when `PlaceAssetBoundState.selected_asset_name` is `Some(name)`:
//! - Mouse cursor shows preview sphere and wireframe bounds
//! - Left click places new asset instance at heightmap intersection
//! - Asset follows mouse position in real-time
//! - Automatically updates instanced renderer with new data
//!
//! ### Selection Mode
//! Active when `PlaceAssetBoundState.selected_asset_name` is `None`:
//! - Left click raycasts against existing asset OBB bounds
//! - Selected assets highlight in red with wireframe colour change
//! - Mouse scroll rotates selected asset around Y axis
//! - Delete key removes selected assets and rebuilds instance buffer
//! - Selected assets follow mouse position for repositioning
//!
//! ## Instance Data Flow
//!
//! ```text
//! PlacedAssetInstance (Component)
//!   └─> Stores per-asset: transform, UV bounds, asset name
//!
//! PlacedAssetInstances (Resource)
//!   └─> Master list for save/load persistence
//!
//! InstancedAssetData (Component)
//!   └─> GPU instance buffer: positions, rotations, UV coords
//!   └─> Automatically extracted to render world
//!   └─> Used by instanced_render_plugin for GPU instancing
//! ```
//!
//! When assets are placed, selected, moved, or deleted, the `RebuildInstancesEvent`
//! triggers `rebuild_instances_on_event()` to synchronise all instance data structures.
//!
//! ## Raycasting
//!
//! Asset selection uses oriented bounding box (OBB) intersection:
//! - Camera ray transformed into asset-local space
//! - AABB slab method tests against half-extents
//! - Closest hit entity selected with depth sorting

/// UI button interactions for asset manager panel (native only).
///
/// Handles collapse toggle, placement activation, and clear all operations.
pub mod interactions;

/// Asset manipulation systems for selection, movement, rotation, and deletion.
///
/// Implements OBB raycasting, scroll-wheel rotation, and instance buffer rebuilding.
pub mod manipulation;

/// Ray intersection utilities for oriented bounding box selection.
///
/// Slab method raycast against transformed AABBs in asset-local space.
pub mod ray;

/// State resources and components for asset manager operation.
///
/// Tracks UI state, placement mode, rotation settings, and placed instances.
pub mod state;

/// UI spawning and update systems for asset manager panel (native only).
///
/// Creates collapsible side panel with placement controls and visual feedback.
pub mod ui;

use bevy::prelude::*;

pub use state::{
    AssetManagerUiState, PlaceAssetBoundState, PlacedAssetInstances, RotationSettings,
};

use ui::{
    apply_collapse_state, reflect_place_cube_button, reflect_selected_asset_label,
    spawn_asset_manager_ui,
};

use manipulation::{
    RebuildInstancesEvent, delete_selected, deselect_on_escape, handle_asset_click,
    manipulate_selected_asset, rebuild_instances_on_event,
};

use interactions::{
    ScrollCapture, clear_bounds_button_interaction, collapse_button_interaction,
    place_cube_button_interaction,
};

// Registers the Asset Manager panel, resources, and systems.
pub struct AssetManagerPlugin;

impl Plugin for AssetManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            // init resources
            .init_resource::<AssetManagerUiState>()
            .init_resource::<PlaceAssetBoundState>()
            .init_resource::<RotationSettings>()
            .init_resource::<PlacedAssetInstances>()
            .init_resource::<ScrollCapture>()
            .add_event::<RebuildInstancesEvent>()
            .add_systems(
                Update,
                (
                    // World
                    delete_selected,
                    manipulate_selected_asset,
                    deselect_on_escape,
                    handle_asset_click,
                    rebuild_instances_on_event,
                ),
            );

        // Add Asset Manager UI only for native builds.
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(
                Update,
                (
                    // Native Only UI
                    collapse_button_interaction,
                    apply_collapse_state,
                    place_cube_button_interaction,
                    reflect_place_cube_button,
                    clear_bounds_button_interaction,
                    reflect_selected_asset_label,
                ),
            );
            app.add_systems(Startup, spawn_asset_manager_ui);
        }
    }
}
