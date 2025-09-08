use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::window::PrimaryWindow;

#[derive(Resource, Default)]
pub struct SelectionBuffer {
    pub selected_ids: Vec<u32>,
}

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct ClassSelectionState {
    pub selection_point: Option<Vec3>,
    pub is_selecting: bool,
}

// Selection buffer system remains unchanged
pub fn update_selection_buffer(
    mut selection_buffer: ResMut<SelectionBuffer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        println!("add selection id");
        selection_buffer.selected_ids.push(2);
    }

    // Number keys 1-9 to set specific IDs
    if keyboard.just_pressed(KeyCode::Digit1) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(10);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(11);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(12);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(13);
    }

    // Clear all selections
    if keyboard.just_pressed(KeyCode::KeyC) {
        selection_buffer.selected_ids.clear();
    }
}

pub fn handle_class_selection(
    mut selection_state: ResMut<ClassSelectionState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut viewport_camera: ResMut<ViewportCamera>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        selection_state.is_selecting = true;
        println!("Click to select");
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        selection_state.is_selecting = false;
        selection_state.selection_point = None;
    }

    let Some(bounds) = assets.get_bounds(&manifests) else {
        return;
    };

    if mouse_button.just_pressed(MouseButton::Left) && selection_state.is_selecting {
        if let (Ok((camera_global_transform, camera)), Ok(window)) =
            (camera_query.single(), windows.single())
        {
            if let Some(cursor_pos) = window.cursor_position() {
                if let Some(world_pos) = viewport_camera.mouse_to_ground_plane(
                    cursor_pos,
                    camera,
                    camera_global_transform,
                    images.get(&assets.heightmap_texture),
                    &bounds,
                ) {
                    selection_state.selection_point =
                        Some(Vec3::new(world_pos.x, world_pos.y, world_pos.z));
                    println!("Selected point: {:?}", selection_state.selection_point);
                }
            }
        }
    }
}
