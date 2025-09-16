use bevy::prelude::*;
use super::state::*;

// Handles interactions for the Asset Manager UI buttons
// Chevron icon toggles collapse state
pub fn collapse_button_interaction(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>, With<CollapseButton>)>,
    mut state: ResMut<AssetManagerUiState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => { state.collapsed = !state.collapsed; *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24)); }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None    => *bg = BackgroundColor(Color::srgb(0.22, 0.24, 0.28)),
        }
    }
}

// Place Cube button toggles placement mode, changes colour when active
pub fn place_cube_button_interaction(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>, With<PlaceCubeButton>)>,
    mut place: ResMut<PlaceAssetBoundState>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => { place.active = !place.active; *bg = BackgroundColor(Color::srgb(0.18, 0.20, 0.24)); }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.26, 0.28, 0.32)),
            Interaction::None    => {
                *bg = BackgroundColor(if place.active { Color::srgb(0.0, 0.90, 0.0) } else { Color::srgb(0.22, 0.24, 0.28) })
            }
        }
    }
}

// Clear All Bounds button despawns all placed bounds and clears instance data
pub fn clear_bounds_button_interaction(
    mut q: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<Button>, With<ClearBoundsButton>)>,
    mut commands: Commands,
    to_clear: Query<Entity, With<PlacedBounds>>,
    existing_instances: Query<Entity, With<crate::engine::render::instanced_render_plugin::InstancedAssetData>>,
    mut placed_assets: ResMut<PlacedAssetInstances>,
) {
    for (interaction, mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed => {
                for e in &to_clear { commands.entity(e).despawn(); }
                for e in &existing_instances { commands.entity(e).despawn(); }
                placed_assets.instances.clear();
                *bg = BackgroundColor(Color::srgb(0.20, 0.12, 0.12));
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.34, 0.14, 0.14)),
            Interaction::None    => *bg = BackgroundColor(Color::srgb(0.28, 0.10, 0.10)),
        }
    }
}
