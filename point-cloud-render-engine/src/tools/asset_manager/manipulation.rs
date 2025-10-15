use super::interactions::ScrollCapture;
use super::ray::ray_hits_obb;
use super::state::*;
use crate::engine::assets::asset_definitions::AssetDefinition;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::render::instanced_render_plugin::{InstanceData, InstancedAssetData};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::mesh::Mesh;
use bevy::render::view::{NoFrustumCulling, RenderLayers};
use bevy::window::PrimaryWindow;
use constants::render_settings::MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE;

#[derive(Component)]
pub struct AssetPreview;

#[derive(Event)]
pub struct RebuildInstancesEvent;

// Unified system: handles both placement of new assets and selection of existing ones
pub fn handle_asset_click(
    buttons: Res<ButtonInput<MouseButton>>,
    place: Res<PlaceAssetBoundState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    viewport_camera: Option<ResMut<ViewportCamera>>,
    q_bounds: Query<(Entity, &GlobalTransform, &BoundsSize, Option<&Selected>), With<PlacedBounds>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventWriter<RebuildInstancesEvent>,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<&mut InstancedAssetData>,
    existing_preview: Query<Entity, With<AssetPreview>>,
) {
    // Clean up preview entities
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn();
    }

    // TODO: (archailect): this is wastefull to run every frame, it should be run only when actually need ie; the state was changed sufficiently.
    // likely all our tools need a Trait that will implement some bind and unbind functionality cleanup and config
    if !place.active {
        for (preview_entity, _, _, _) in q_bounds.iter() {
            commands.entity(preview_entity).insert(Visibility::Hidden);
        }
        return;
    } else {
        // Make bounds visible when placement mode is active
        for (entity, _, _, _) in q_bounds.iter() {
            commands.entity(entity).insert(Visibility::Visible);
        }
    }

    let clicked = buttons.just_pressed(MouseButton::Left);

    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((cam_xf, camera)) = cameras.single() else {
        return;
    };

    // PLACEMENT MODE: selected_asset_name is Some
    if let Some(ref asset_name) = place.selected_asset_name {
        // Validate prerequisites for placement
        let Some(scene_bounds) = assets.get_bounds(&manifests) else {
            return;
        };
        let Some(height_img) = images.get(&assets.heightmap_texture) else {
            return;
        };
        let Some(mut viewport_camera) = viewport_camera else {
            return;
        };

        // Raycast to ground
        let hit = viewport_camera.mouse_to_ground_plane(
            cursor_pos,
            camera,
            cam_xf,
            Some(height_img),
            &scene_bounds,
        );
        let Some(hit) = hit else { return };

        // Get selected asset metadata
        let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) else {
            return;
        };
        let asset_meta = manifest
            .asset_atlas
            .as_ref()
            .and_then(|aa| aa.assets.iter().find(|a| a.name == *asset_name));
        let Some(asset_meta) = asset_meta else { return };

        let size = calculate_asset_size(asset_meta);
        let center = Vec3::new(hit.x, hit.y + size.y * 0.5, hit.z);

        // Show preview at mouse position
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(hit),
            AssetPreview,
            RenderLayers::layer(1),
        ));

        // Spawn preview bounds
        commands.spawn((
            create_wireframe_mesh_bundle(&mut meshes, &mut materials, size, center),
            AssetPreview,
            bevy::render::view::NoIndirectDrawing,
            NoFrustumCulling,
            Name::new(format!("{}_preview_bounds", asset_meta.name)),
        ));

        // On click, place the asset (don't select it)
        if clicked {
            let transform = Transform::from_translation(center);
            let uv_bounds = Vec4::new(
                asset_meta.uv_bounds.uv_min[0],
                asset_meta.uv_bounds.uv_min[1],
                asset_meta.uv_bounds.uv_max[0],
                asset_meta.uv_bounds.uv_max[1],
            );

            let placed_instance = PlacedAssetInstance {
                asset_name: asset_meta.name.clone(),
                transform,
                uv_bounds,
            };
            placed_assets.instances.push(placed_instance.clone());

            // Spawn the actual bounds entity (not selected, not preview)
            commands.spawn((
                create_wireframe_mesh_bundle(&mut meshes, &mut materials, size, center),
                placed_instance.clone(),
                PlacedBounds,
                BoundsSize(size),
                bevy::render::view::NoIndirectDrawing,
                NoFrustumCulling,
                Name::new(format!("{}_bounds_wire", asset_meta.name)),
            ));

            // Update instanced renderer
            if let Ok(mut data) = existing_instances.single_mut() {
                // update_instance_data(&mut data, &placed_assets.instances, asset_meta);
            } else {
                create_new_instanced_renderer(
                    &mut commands,
                    &mut meshes,
                    &placed_assets.instances,
                    asset_meta,
                );
            }
        }

        events.write(RebuildInstancesEvent);
        return;
    }

    // SELECTION MODE: selected_asset_name is None
    if clicked {
        // Check if we clicked on an existing asset
        if let Ok(ray) = camera.viewport_to_world(cam_xf, cursor_pos) {
            let origin = ray.origin;
            let dir = ray.direction.as_vec3();

            let mut best_hit: Option<(Entity, f32, bool)> = None;
            for (e, xf, BoundsSize(size), selected) in &q_bounds {
                if let Some(t) = ray_hits_obb(origin, dir, *xf, *size) {
                    if t > 0.0 && (best_hit.is_none() || t < best_hit.unwrap().1) {
                        best_hit = Some((e, t, selected.is_some()));
                    }
                }
            }

            // If we hit an asset, handle selection
            if let Some((hit_e, _, was_selected)) = best_hit {
                deselect_all(
                    q_bounds.iter().map(|(e, _gt, _bs, sel)| (e, sel)),
                    &mut commands,
                );

                // Toggle selection: if it was already selected, deselect it; otherwise select it
                if !was_selected {
                    select_asset(&mut commands, hit_e);
                }
                return;
            }
        }

        // Clicked on empty space, deselect all
        deselect_all(
            q_bounds.iter().map(|(e, _gt, _bs, sel)| (e, sel)),
            &mut commands,
        );
    }
}

pub fn rebuild_instances_on_event(
    mut events: EventReader<RebuildInstancesEvent>,
    q_selected: Query<(&mut Transform, &mut PlacedAssetInstance, &BoundsSize), With<Selected>>,
    q_unselected: Query<&PlacedAssetInstance, (With<PlacedBounds>, Without<Selected>)>,
    mut existing_instances: Query<&mut InstancedAssetData>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
) {
    for _event in events.read() {
        rebuild_instances_from_both(
            &q_selected,
            &q_unselected,
            &mut existing_instances,
            &manifests,
            &assets,
        );
    }
}

fn rebuild_instances_from_both(
    q_selected: &Query<(&mut Transform, &mut PlacedAssetInstance, &BoundsSize), With<Selected>>,
    q_unselected: &Query<&PlacedAssetInstance, (With<PlacedBounds>, Without<Selected>)>,
    existing_instances: &mut Query<&mut InstancedAssetData>,
    manifests: &Res<Assets<SceneManifest>>,
    assets: &Res<PointCloudAssets>,
) {
    if let Ok(mut instance_data) = existing_instances.single_mut() {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(asset_meta) = manifest
                .asset_atlas
                .as_ref()
                .and_then(|aa| aa.assets.first())
            {
                instance_data.0 = q_selected
                    .iter()
                    .map(|(_, placed, _)| placed)
                    .chain(q_unselected.iter())
                    .map(|placed| InstanceData {
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
                    })
                    .collect();
            }
        }
    }
}

// Unified manipulation: moves and rotates selected assets
pub fn manipulate_selected_asset(
    mut wheel: EventReader<MouseWheel>,
    mut q: Query<(&mut Transform, &mut PlacedAssetInstance, &BoundsSize), With<Selected>>,
    q_all_placed: Query<&PlacedAssetInstance, (With<PlacedBounds>, Without<Selected>)>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    viewport_camera: Option<ResMut<ViewportCamera>>,
    settings: Res<RotationSettings>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    mut existing_instances: Query<&mut InstancedAssetData>,
    mut cap: ResMut<ScrollCapture>,
    time: Res<Time>,
) {
    if q.is_empty() {
        cap.lock_zoom_this_frame = false;
        return;
    }

    let mut needs_update = false;

    // Handle rotation via scroll wheel
    if !wheel.is_empty() {
        cap.lock_zoom_this_frame = true;

        let mut delta = 0.0f32;
        for ev in wheel.read() {
            delta += ev.y;
        }

        if delta.abs() >= f32::EPSILON {
            for (mut transform, mut placed_instance, _) in &mut q {
                let angle = delta * settings.speed * time.delta_secs();
                transform.rotate_y(angle);
                placed_instance.transform = *transform;
            }
            needs_update = true;
        }
    }

    // Handle position following mouse
    let Ok(window) = windows.single() else { return };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(mut viewport_camera) = viewport_camera else {
        return;
    };
    let Ok((cam_xform, camera)) = cameras.single() else {
        return;
    };
    let Some(scene_bounds) = assets.get_bounds(&manifests) else {
        return;
    };
    let Some(height_img) = images.get(&assets.heightmap_texture) else {
        return;
    };

    let hit = viewport_camera.mouse_to_ground_plane(
        cursor_pos,
        camera,
        cam_xform,
        Some(height_img),
        &scene_bounds,
    );
    let Some(hit) = hit else { return };
    for (mut transform, mut placed_instance, BoundsSize(size)) in &mut q {
        let new_center = Vec3::new(hit.x, hit.y + size.y * 0.5, hit.z);
        if transform.translation.distance(new_center) > 0.001 {
            transform.translation = new_center;
            placed_instance.transform = *transform;
            needs_update = true;
        }
    }

    if needs_update {
        rebuild_instances_from_both(
            &q,
            &q_all_placed,
            &mut existing_instances,
            &manifests,
            &assets,
        );
    }
}

// Deselect on Escape or clicking empty space is handled via handle_asset_click
pub fn deselect_on_escape(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_bounds: Query<(Entity, Option<&Selected>), With<PlacedBounds>>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        deselect_all(q_bounds.iter(), &mut commands);
    }
}

// Delete selected assets with Delete key
pub fn delete_selected(
    keyboard: Res<ButtonInput<KeyCode>>,
    q_bounds: Query<(Entity, &PlacedAssetInstance), (With<PlacedBounds>, With<Selected>)>,
    q_all_placed: Query<&PlacedAssetInstance, (With<PlacedBounds>, Without<Selected>)>,
    mut commands: Commands,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<(Entity, &mut InstancedAssetData)>,
    place: Res<PlaceAssetBoundState>,
    manifests: Res<Assets<SceneManifest>>,
    assets: Res<PointCloudAssets>,
) {
    if !keyboard.just_pressed(KeyCode::Delete) && place.active || q_bounds.is_empty() {
        return;
    }

    let to_delete: Vec<_> = q_bounds.iter().map(|(e, inst)| (e, inst.clone())).collect();

    // Despawn the bound entities
    for (entity, _) in &to_delete {
        commands.entity(*entity).despawn();
    }

    // Update the resource list (for save/load purposes)
    for (_, instance) in &to_delete {
        placed_assets.instances.retain(|inst| {
            !(inst.asset_name == instance.asset_name
                && inst
                    .transform
                    .translation
                    .distance(instance.transform.translation)
                    < 0.1)
        });
    }

    // Rebuild instance data from remaining components (which excludes deleted ones after despawn)
    // Since we just despawned, we need to collect from what's left
    rebuild_instances(
        q_all_placed,
        existing_instances,
        commands,
        assets,
        manifests,
    );
}

fn rebuild_instances(
    q_all_placed: Query<&PlacedAssetInstance, (With<PlacedBounds>, Without<Selected>)>,
    mut existing_instances: Query<(Entity, &mut InstancedAssetData)>,
    mut commands: Commands,
    assets: Res<PointCloudAssets>,
    manifests: Res<Assets<SceneManifest>>,
) {
    // Rebuild instance data from remaining components (which excludes deleted ones after despawn)
    // Since we just despawned, we need to collect from what's left
    if q_all_placed.is_empty() {
        // No assets left, despawn the renderer
        for (entity, _) in existing_instances.iter() {
            commands.entity(entity).despawn();
        }
    } else if let Ok((_, mut instance_data)) = existing_instances.single_mut() {
        if let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) {
            if let Some(asset_meta) = manifest
                .asset_atlas
                .as_ref()
                .and_then(|aa| aa.assets.first())
            {
                // Rebuild from the unselected (remaining) components only
                instance_data.0 = q_all_placed
                    .iter()
                    .map(|placed| InstanceData {
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
                    })
                    .collect();
            }
        }
    }
}

fn select_asset(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert(Selected);
    commands.entity(entity).insert(ActiveRotating);
    commands
        .entity(entity)
        .insert(bevy::pbr::wireframe::WireframeColor {
            color: Color::srgb(1.0, 0.0, 0.0),
        });
}

fn deselect_all<'a>(
    items: impl Iterator<Item = (Entity, Option<&'a Selected>)>,
    commands: &mut Commands,
) {
    for (e, sel) in items {
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

fn calculate_asset_size(asset_meta: &AssetDefinition) -> Vec3 {
    let lb = &asset_meta.local_bounds;
    let mut sx = (lb.max_x - lb.min_x).max(0.001) as f32;
    let mut sy = (lb.max_y - lb.min_y).max(0.001) as f32;
    let mut sz = (lb.max_z - lb.min_z).max(0.001) as f32;

    if !sx.is_finite() {
        sx = 0.001;
    }
    if !sy.is_finite() {
        sy = 0.001;
    }
    if !sz.is_finite() {
        sz = 0.001;
    }

    Vec3::new(sx, sy, sz)
}

fn create_wireframe_mesh_bundle(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    size: Vec3,
    center: Vec3,
) -> (Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::LineList,
        bevy::asset::RenderAssetUsages::default(),
    );

    let half = size / 2.0;
    let vertices = vec![
        [-half.x, -half.y, -half.z],
        [half.x, -half.y, -half.z],
        [half.x, -half.y, -half.z],
        [half.x, -half.y, half.z],
        [half.x, -half.y, half.z],
        [-half.x, -half.y, half.z],
        [-half.x, -half.y, half.z],
        [-half.x, -half.y, -half.z],
        [-half.x, half.y, -half.z],
        [half.x, half.y, -half.z],
        [half.x, half.y, -half.z],
        [half.x, half.y, half.z],
        [half.x, half.y, half.z],
        [-half.x, half.y, half.z],
        [-half.x, half.y, half.z],
        [-half.x, half.y, -half.z],
        [-half.x, -half.y, -half.z],
        [-half.x, half.y, -half.z],
        [half.x, -half.y, -half.z],
        [half.x, half.y, -half.z],
        [half.x, -half.y, half.z],
        [half.x, half.y, half.z],
        [-half.x, -half.y, half.z],
        [-half.x, half.y, half.z],
    ];

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32((0u32..24).collect()));

    let material = StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        emissive: Color::srgba(0.0, 0.0, 0.0, 0.0).into(),
        perceptual_roughness: 1.0,
        ..default()
    };

    (
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(center),
    )
}

fn create_new_instanced_renderer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    instances: &[PlacedAssetInstance],
    asset_meta: &AssetDefinition,
) {
    if instances.is_empty() {
        return;
    }

    let instance_data: Vec<InstanceData> = instances
        .iter()
        .map(|placed| InstanceData {
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
        })
        .collect();

    commands.spawn((
        Mesh3d(meshes.add(
            crate::engine::mesh::point_index_mesh::create_point_index_mesh(asset_meta.point_count),
        )),
        InstancedAssetData(instance_data),
        Transform::IDENTITY,
        NoFrustumCulling,
        bevy::render::view::NoIndirectDrawing,
        Name::new("InstancedAssetRenderer"),
    ));
}
