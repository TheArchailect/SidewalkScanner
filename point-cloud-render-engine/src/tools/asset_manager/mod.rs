pub mod interactions;
pub mod placement;
pub mod ray;
pub mod rotation;
pub mod selection;
pub mod state;
pub mod ui;

use bevy::prelude::*;

// Re-export state types for other modules
pub use state::{
    AssetManagerUiState, PlaceAssetBoundState, PlacedAssetInstances, RotationSettings,
    SelectionLock,
};

// Import UI-related systems
use ui::{
    apply_collapse_state, reflect_place_cube_button, reflect_selected_asset_label,
    spawn_asset_manager_ui,
};

// Import button interaction systems
use interactions::{
    ScrollCapture, clear_bounds_button_interaction, collapse_button_interaction,
    place_cube_button_interaction, reset_scroll_capture,
};
use placement::place_cube_on_world_click;
use rotation::rotate_active_bounds_on_scroll;
use selection::{deselect_on_escape, reflect_selection_lock, toggle_select_on_click};

// Registers the Asset Manager panel, resources, and systems.
pub struct AssetManagerUiPlugin;

impl Plugin for AssetManagerUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // init resources
            .init_resource::<AssetManagerUiState>()
            .init_resource::<PlaceAssetBoundState>()
            .init_resource::<RotationSettings>()
            .init_resource::<SelectionLock>()
            .init_resource::<PlacedAssetInstances>()
            .init_resource::<ScrollCapture>()
            .add_systems(
                Update,
                (
                    // UI
                    collapse_button_interaction,
                    apply_collapse_state,
                    place_cube_button_interaction,
                    reflect_place_cube_button,
                    clear_bounds_button_interaction,
                    reflect_selected_asset_label,
                    // World
                    place_cube_on_world_click,
                    toggle_select_on_click,
                    rotate_active_bounds_on_scroll,
                    reflect_selection_lock,
                    deselect_on_escape,
                    reset_scroll_capture,
                ),
            );

        // Add Asset Manager UI only for native builds.
        #[cfg(not(target_arch = "wasm32"))]
        {
            app.add_systems(Startup, spawn_asset_manager_ui);
        }
    }
}
