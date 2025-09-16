pub mod state;
pub mod ui;
pub mod interactions;
pub mod placement;
pub mod selection;
pub mod rotation;
pub mod ray;

use bevy::prelude::*;

// Re-export state types for other modules
pub use state::{
    AssetManagerUiState, 
    PlaceAssetBoundState,
    PlacedAssetInstances,
    RotationSettings,
    SelectionLock,
};

// Import UI-related systems
use ui::{
    spawn_asset_manager_ui,
    apply_collapse_state,
    reflect_place_cube_button,
    reflect_selected_asset_label,
};

// Import button interaction systems
use interactions::{
    collapse_button_interaction,
    place_cube_button_interaction,
    clear_bounds_button_interaction,
};
use placement::place_cube_on_world_click;
use selection::{toggle_select_on_click, reflect_selection_lock};
use rotation::rotate_active_bounds_on_scroll;

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
            // UI
            .add_systems(Startup, spawn_asset_manager_ui)
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
                ),
            );
    }
}

