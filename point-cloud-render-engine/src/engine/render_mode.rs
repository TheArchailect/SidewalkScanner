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
    ConnectivityClass = 6,
}

/// Handle render mode switching via keyboard input.
/// Mode changes trigger compute shader recomputation via resource change detection.
/// Custom render pipeline receives mode data through RenderModeState extraction.
pub fn render_mode_system(
    mut render_state: ResMut<RenderModeState>,
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

    if keyboard.just_pressed(KeyCode::KeyM) {
        new_mode = RenderMode::ConnectivityClass;
        mode_changed = true;
        println!("Render mode: Connectivity Class");
    }

    if mode_changed {
        render_state.current_mode = new_mode;
        // Resource change detection triggers compute shader recomputation.
        // Custom render pipeline receives updated mode through extraction system.
    }
}
