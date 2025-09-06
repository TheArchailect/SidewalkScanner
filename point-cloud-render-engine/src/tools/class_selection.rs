use crate::SceneManifest;
use crate::engine::camera::ViewportCamera;
use crate::engine::point_cloud::PointCloudAssets;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::window::PrimaryWindow;
#[derive(Resource, Default, ExtractResource, Clone)]
pub struct ClassSelectionState {
    pub selection_point: Option<Vec3>,
    pub is_selecting: bool,
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
