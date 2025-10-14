pub mod interactions;
pub mod manipulation;
pub mod ray;
pub mod state;
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
    delete_selected, deselect_on_escape, handle_asset_click, manipulate_selected_asset,
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
            .add_systems(
                Update,
                (
                    // World
                    delete_selected,
                    deselect_on_escape,
                    manipulate_selected_asset,
                    handle_asset_click,
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
