use crate::engine::point_cloud::PointCloud;
use crate::engine::shaders::{PointCloudShader, RenderMode};
/// Rendering mode control system
use bevy::prelude::*;

#[derive(Resource)]
pub struct RenderModeState {
    pub current_mode: RenderMode,
}

impl Default for RenderModeState {
    fn default() -> Self {
        Self {
            current_mode: RenderMode::RgbColour,
        }
    }
}

/// Handle render mode switching via keyboard
pub fn render_mode_system(
    mut render_state: ResMut<RenderModeState>,
    mut materials: ResMut<Assets<PointCloudShader>>,
    material_query: Query<&MeshMaterial3d<PointCloudShader>, With<PointCloud>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let mut mode_changed = false;
    let mut new_mode = render_state.current_mode;

    if keyboard.just_pressed(KeyCode::KeyR) {
        new_mode = RenderMode::RgbColour;
        mode_changed = true;
        println!("Render mode: RGB Colour");
    }

    if keyboard.just_pressed(KeyCode::KeyO) {
        new_mode = RenderMode::OriginalClassification;
        mode_changed = true;
        println!("Render mode: Original Classification");
    }

    if keyboard.just_pressed(KeyCode::KeyM) {
        new_mode = RenderMode::ModifiedClassification;
        mode_changed = true;
        println!("Render mode: Modified Classification");
    }

    if mode_changed {
        render_state.current_mode = new_mode;

        // Update shader uniform
        for material_handle in material_query.iter() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.polygon_data.render_mode = new_mode as u32;
            }
        }
    }
}
