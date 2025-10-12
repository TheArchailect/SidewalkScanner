use super::ray::ray_hits_obb;
use super::state::*;
use bevy::prelude::*;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::render::instanced_render_plugin::InstancedAssetData;

// Toggles selection of placed bounds on left mouse click
pub fn toggle_select_on_click(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    q_bounds: Query<(Entity, &GlobalTransform, &BoundsSize, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((cam_xf, camera)) = cameras.single() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(cam_xf, cursor_pos) else {
        return;
    };
    let origin = ray.origin;
    let dir = ray.direction.as_vec3();

    let mut best: Option<(Entity, f32, bool)> = None;
    for (e, xf, BoundsSize(size), selected) in &q_bounds {
        if let Some(t) = ray_hits_obb(origin, dir, *xf, *size) {
            if t > 0.0 && (best.is_none() || t < best.unwrap().1) {
                best = Some((e, t, selected.is_some()));
            }
        }
    }

    if let Some((hit_e, _t, was_selected)) = best {
        // Deselect all
        for (e, _, _, sel) in &q_bounds {
            if sel.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands
                    .entity(e)
                    .insert(bevy::pbr::wireframe::WireframeColor {
                        color: Color::WHITE,
                    });
            }
        }
        // Select hit
        if !was_selected {
            commands.entity(hit_e).insert(Selected);
            commands.entity(hit_e).insert(ActiveRotating);
            commands
                .entity(hit_e)
                .insert(bevy::pbr::wireframe::WireframeColor {
                    color: Color::srgb(1.0, 1.0, 0.0),
                });
        }
    } else {
        for (e, _, _, selected) in &q_bounds {
            if selected.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands
                    .entity(e)
                    .insert(bevy::pbr::wireframe::WireframeColor {
                        color: Color::WHITE,
                    });
            }
        }
    }
}

pub fn reflect_selection_lock(
    q_selected: Query<(), With<Selected>>,
    mut lock: ResMut<SelectionLock>,
) {
    lock.active = !q_selected.is_empty();
}

// Deselect all on Escape key press
pub fn deselect_on_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_bounds: Query<(Entity, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        for (e, sel) in &q_bounds {
            if sel.is_some() {
                commands.entity(e).remove::<Selected>();
                commands.entity(e).remove::<ActiveRotating>();
                commands
                    .entity(e)
                    .insert(bevy::pbr::wireframe::WireframeColor {
                        color: Color::WHITE,
                    });
            }
        }
    }
}

// Delete selected assets with Delete key
pub fn delete_selected_on_delete(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_bounds: Query<(Entity, &PlacedAssetInstance), (With<PlacedBounds>, With<Selected>)>,
    mut commands: Commands,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<(Entity, &mut InstancedAssetData)>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
) {
    if !keyboard.just_pressed(KeyCode::Delete) {
        return;
    }

    if q_bounds.is_empty() {
        return;
    }

    let mut to_delete = Vec::new();
    for (entity, placed_instance) in &q_bounds {
        to_delete.push((entity, placed_instance.clone()));
    }

    for (entity, _) in &to_delete {
        commands.entity(*entity).despawn();
    }

    for (_, instance_to_remove) in &to_delete {
        placed_assets.instances.retain(|inst| {
            !(inst.asset_name == instance_to_remove.asset_name
                && inst.transform.translation.distance(instance_to_remove.transform.translation) < 0.1)
        });
    }

    if placed_assets.instances.is_empty() {
        
        for (entity, _) in existing_instances.iter() {
            commands.entity(entity).despawn();
        }
    } else {
        if let Ok((_, mut instance_data)) = existing_instances.get_single_mut() {
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
}
