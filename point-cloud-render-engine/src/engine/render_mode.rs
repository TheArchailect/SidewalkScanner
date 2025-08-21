use crate::engine::point_cloud::PointCloud;
use crate::engine::shaders::PointCloudShader;
/// Rendering mode control system
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
#[derive(Resource, Clone, ExtractResource)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    OriginalClassification = 0,
    ModifiedClassification = 1,
    RgbColour = 2,
    MortonCode = 3,
    PerformanceDebug = 4,
    ClassSelection = 5,
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

    if keyboard.just_pressed(KeyCode::KeyZ) {
        new_mode = RenderMode::RgbColour;
        mode_changed = true;
        println!("Render mode: RGB Colour");
    }

    if keyboard.just_pressed(KeyCode::KeyX) {
        new_mode = RenderMode::OriginalClassification;
        mode_changed = true;
        println!("Render mode: Original Classification");
    }

    if keyboard.just_pressed(KeyCode::KeyC) {
        new_mode = RenderMode::ModifiedClassification;
        mode_changed = true;
        println!("Render mode: Modified Classification");
    }

    if keyboard.just_pressed(KeyCode::KeyV) {
        new_mode = RenderMode::MortonCode;
        mode_changed = true;
        println!("Render mode: Morton Code");
    }

    if keyboard.just_pressed(KeyCode::KeyB) {
        new_mode = RenderMode::PerformanceDebug;
        mode_changed = true;
        println!("Render mode: Performance Debug");
    }

    if keyboard.just_pressed(KeyCode::KeyN) {
        new_mode = RenderMode::ClassSelection;
        mode_changed = true;
        println!("Render mode: Class Selection");
    }

    if mode_changed {
        render_state.current_mode = new_mode;
        // Compute shader will detect RenderModeState.is_changed() and recompute
    }
}
