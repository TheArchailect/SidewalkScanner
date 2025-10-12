use super::interactions::ScrollCapture;
use super::state::*;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::render::instanced_render_plugin::InstancedAssetData;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Helper to update instanced renderer data
fn update_instance(
    placed_assets: &PlacedAssetInstances,
    existing_instances: &mut Query<&mut InstancedAssetData>,
    manifests: &Res<Assets<SceneManifest>>,
    assets: &Res<PointCloudAssets>,
) {
    if let Ok(mut instance_data) = existing_instances.single_mut() {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(asset_meta) = manifest.asset_atlas.as_ref().and_then(|aa| aa.assets.first()) {
                instance_data.0 = placed_assets.instances.iter().map(|placed| {
                    crate::engine::render::instanced_render_plugin::InstanceData {
                        position: placed.transform.translation.to_array(),
                        _padding1: 0.0,
                        rotation: [
                            placed.transform.rotation.x,
                            placed.transform.rotation.y,
                            placed.transform.rotation.z,
                            placed.transform.rotation.w,
                        ],
                        uv_bounds: placed.uv_bounds.to_array(),
                        point_count: asset_meta.point_count as f32,
                        _padding2: [0.0; 3],
                    }
                }).collect();
            }
        }
    }
}

// Rotates any actively selected bounds on mouse wheel scroll
pub fn rotate_active_bounds_on_scroll(
    mut wheel: EventReader<MouseWheel>,
    mut q: Query<
        (&mut Transform, &mut PlacedAssetInstance),
        (With<ActiveRotating>, With<Selected>),
    >,
    settings: Res<RotationSettings>,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<&mut InstancedAssetData>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
    mut cap: ResMut<ScrollCapture>,
    time: Res<Time>,
) {
    if q.is_empty() {
        return;
    }
    let mut delta = 0.0f32;
    for ev in wheel.read() {
        delta += ev.y as f32;
    }
    if delta.abs() < f32::EPSILON {
        return;
    }
    cap.lock_zoom_this_frame = true;

    for (mut transform, mut placed_instance) in &mut q {
        let angle = delta * settings.speed * time.delta_secs() * 4.0; // change to adjust rotation speed
        transform.rotate_y(angle);
        placed_instance.transform = *transform;

        if let Some(global_instance) = placed_assets.instances.iter_mut().find(|inst| {
            inst.asset_name == placed_instance.asset_name
                && inst
                    .transform
                    .translation
                    .distance(placed_instance.transform.translation)
                    < 0.1
        }) {
            global_instance.transform = *transform;
        }
    }

    update_instance(&placed_assets, &mut existing_instances, &manifests, &assets);
}

// Moves selected assets to follow mouse position
pub fn drag_move_active_bounds(
    mut q: Query<(&mut Transform, &mut PlacedAssetInstance, &BoundsSize), With<Selected>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    viewport_camera: Option<ResMut<ViewportCamera>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<&mut InstancedAssetData>,
) {
    if q.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else { return; };
    let Some(cursor_pos) = window.cursor_position() else { return; };
    let Some(mut viewport_camera) = viewport_camera else { return; };
    let Ok((cam_xform, camera)) = cameras.single() else { return; };
    let Some(scene_bounds) = assets.get_bounds(&manifests) else { return; };
    let Some(height_img) = images.get(&assets.heightmap_texture) else { return; };

    let hit = viewport_camera.mouse_to_ground_plane(cursor_pos, camera, cam_xform, Some(height_img), &scene_bounds);
    let Some(hit) = hit else { return; };

    for (mut transform, mut placed_instance, size) in &mut q {
        let old_pos = transform.translation;
        let new_center = Vec3::new(hit.x, hit.y + size.0.y * 0.5, hit.z);
        transform.translation = new_center;
        placed_instance.transform = *transform;

        if let Some(global_instance) = placed_assets.instances.iter_mut().find(|inst| {
            inst.asset_name == placed_instance.asset_name
                && inst.transform.translation.distance(old_pos) < 0.1
        }) {
            global_instance.transform = *transform;
        }
    }

    update_instance(&placed_assets, &mut existing_instances, &manifests, &assets);
}