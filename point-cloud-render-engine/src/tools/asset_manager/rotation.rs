use super::interactions::ScrollCapture;
use super::state::*;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::render::instanced_render_plugin::InstancedAssetData;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

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
        let angle = delta * settings.speed * time.delta_secs();
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

    if let Ok(mut instance_data) = existing_instances.single_mut() {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(asset_meta) = manifest
                .asset_atlas
                .as_ref()
                .and_then(|aa| aa.assets.first())
            {
                let new_data: Vec<crate::engine::render::instanced_render_plugin::InstanceData> =
                    placed_assets
                        .instances
                        .iter()
                        .map(|placed| {
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
                        })
                        .collect();
                instance_data.0 = new_data;
            }
        }
    }
}
