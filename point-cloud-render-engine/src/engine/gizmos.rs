use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::camera::ViewportCamera;
use super::point_cloud::PointCloudAssets;

#[derive(Component)]
pub struct MouseIntersectionGizmo;

#[derive(Component)]
pub struct DirectionGizmo;

pub fn create_direction_gizmo(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("./textures/arrow.png")),
            unlit: true,
            cull_mode: None,
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        DirectionGizmo,
    ));
}

pub fn update_direction_gizmo(
    mut gizmo_query: Query<&mut Transform, (With<DirectionGizmo>, Without<Camera3d>)>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: ResMut<ViewportCamera>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
) {
    if let (Ok(mut gizmo_transform), Ok((camera_global_transform, camera))) =
        (gizmo_query.single_mut(), camera_query.single())
    {
        let window = windows.single();

        if let Some(cursor_pos) = window.unwrap().cursor_position() {
            if let Some(intersection) = maps_camera.mouse_to_ground_plane(
                cursor_pos,
                camera,
                camera_global_transform,
                images.get(&assets.heightmap_texture),
                assets.bounds.as_ref(),
            ) {
                gizmo_transform.translation =
                    Vec3::new(intersection.x, intersection.y + 1.0, intersection.z);

                let focus_to_mouse = intersection - maps_camera.focus_point;
                let movement_direction =
                    Vec3::new(-focus_to_mouse.x, 0.0, focus_to_mouse.z).normalize();
                let angle =
                    movement_direction.z.atan2(movement_direction.x) - std::f32::consts::FRAC_PI_2;

                gizmo_transform.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                    * Quat::from_rotation_z(angle);
            }
        }
    }
}

pub fn update_mouse_intersection_gizmo(
    mut gizmo_query: Query<(&mut Transform, &mut Visibility), With<MouseIntersectionGizmo>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut maps_camera: ResMut<ViewportCamera>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
) {
    if let (
        Ok((mut gizmo_transform, mut gizmo_visibility)),
        Ok((camera_global_transform, camera)),
    ) = (gizmo_query.single_mut(), camera_query.single())
    {
        let window = windows.single();

        if let Some(cursor_pos) = window.unwrap().cursor_position() {
            if let Some(intersection) = maps_camera.mouse_to_ground_plane(
                cursor_pos,
                camera,
                camera_global_transform,
                images.get(&assets.heightmap_texture),
                assets.bounds.as_ref(),
            ) {
                gizmo_transform.translation = intersection;
                *gizmo_visibility = Visibility::Visible;
            } else {
                *gizmo_visibility = Visibility::Hidden;
            }
        } else {
            *gizmo_visibility = Visibility::Hidden;
        }
    }
}
