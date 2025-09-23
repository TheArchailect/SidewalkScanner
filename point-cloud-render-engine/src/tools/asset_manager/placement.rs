use crate::engine::assets::asset_definitions::AssetDefinition;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::render::instanced_render_plugin::{InstanceData, InstancedAssetData};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::view::NoFrustumCulling;

use super::state::*;
use bevy::prelude::{Mesh3d, MeshMaterial3d};
use bevy::render::mesh::Mesh;
use bevy::window::PrimaryWindow;

// Click in world to place bounds & update instanced renderer
pub fn place_cube_on_world_click(
    buttons: Res<ButtonInput<MouseButton>>,
    place: Res<PlaceAssetBoundState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    maps_camera: Option<ResMut<ViewportCamera>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    manifests: Res<Assets<SceneManifest>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut placed_assets: ResMut<PlacedAssetInstances>,
    mut existing_instances: Query<&mut InstancedAssetData>,
) {
    // Only run if placement mode is active and right mouse was just clicked
    if !place.active || !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Validate prereqs (camera, window, cursor pos, scene bounds, heightmap, etc)
    let Some(mut maps_camera) = maps_camera else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
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

    // Raycast from mouse to ground plane
    let hit = maps_camera.mouse_to_ground_plane(
        cursor_pos,
        camera,
        cam_xform,
        Some(height_img),
        &scene_bounds,
    );
    let Some(hit) = hit else {
        return;
    };

    // Lookup which asset is currently selected in the manifest
    let Some(manifest) = assets.manifest.as_ref().and_then(|h| manifests.get(h)) else {
        return;
    };

    let picked = if let Some(ref name) = place.selected_asset_name {
        manifest
            .asset_atlas
            .as_ref()
            .and_then(|aa| aa.assets.iter().find(|a| a.name == *name))
    } else {
        return;
    };

    // we shouldn't have a default chosen asset for placement
    // else {
    //     manifest
    //         .asset_atlas
    //         .as_ref()
    //         .and_then(|aa| aa.assets.first())
    // };

    let Some(asset_meta) = picked else {
        return;
    };

    // Bounds size from metadata
    let lb = &asset_meta.local_bounds;
    let mut sx = (lb.max_x - lb.min_x) as f32;
    let mut sy = (lb.max_y - lb.min_y) as f32;
    let mut sz = (lb.max_z - lb.min_z) as f32;
    if !sx.is_finite() || !sy.is_finite() || !sz.is_finite() {
        return;
    }
    if sx <= 0.0 {
        sx = 0.001;
    }
    if sy <= 0.0 {
        sy = 0.001;
    }
    if sz <= 0.0 {
        sz = 0.001;
    }
    let size = Vec3::new(sx, sy, sz);

    // Centre cuboid so it sits flat on ground (centre offset by half height)
    let center = Vec3::new(hit.x, hit.y + size.y * 0.5, hit.z);
    let transform = Transform::from_translation(center);

    // Grab UV bounds for this asset for instancing
    let uv_bounds = Vec4::new(
        asset_meta.uv_bounds.uv_min[0],
        asset_meta.uv_bounds.uv_min[1],
        asset_meta.uv_bounds.uv_max[0],
        asset_meta.uv_bounds.uv_max[1],
    );

    // Build data for placed instance and add to resource list
    let placed_instance = PlacedAssetInstance {
        asset_name: asset_meta.name.clone(),
        transform,
        uv_bounds,
    };
    placed_assets.instances.push(placed_instance.clone());

    commands.spawn((
        // Create wireframe using 12 line segments for cube edges
        create_wireframe_mesh_bundle(&mut meshes, &mut materials, size, center),
        placed_instance.clone(),
        PlacedBounds,
        BoundsSize(size),
        bevy::render::view::NoIndirectDrawing,
        NoFrustumCulling,
        Name::new(format!("{}_bounds_wire", asset_meta.name)),
    ));

    // Update or create instanced renderer
    if let Ok(mut data) = existing_instances.single_mut() {
        update_instance_data(&mut data, &placed_assets.instances, asset_meta);
    } else {
        create_new_instanced_renderer(
            &mut commands,
            &mut meshes,
            &placed_assets.instances,
            asset_meta,
        );
    }
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
        // Bottom face edges
        [-half.x, -half.y, -half.z],
        [half.x, -half.y, -half.z],
        [half.x, -half.y, -half.z],
        [half.x, -half.y, half.z],
        [half.x, -half.y, half.z],
        [-half.x, -half.y, half.z],
        [-half.x, -half.y, half.z],
        [-half.x, -half.y, -half.z],
        // Top face edges
        [-half.x, half.y, -half.z],
        [half.x, half.y, -half.z],
        [half.x, half.y, -half.z],
        [half.x, half.y, half.z],
        [half.x, half.y, half.z],
        [-half.x, half.y, half.z],
        [-half.x, half.y, half.z],
        [-half.x, half.y, -half.z],
        // Vertical edges
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
        base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
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

// update instanced renderer data on new placement
fn update_instance_data(
    instance_data: &mut InstancedAssetData,
    instances: &[PlacedAssetInstance],
    asset_meta: &AssetDefinition,
) {
    let new_data: Vec<InstanceData> = instances
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

    instance_data.0 = new_data;
}

// Helper function to create new instanced renderer
fn create_new_instanced_renderer(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    instances: &[PlacedAssetInstance],
    asset_meta: &AssetDefinition,
) {
    if instances.is_empty() {
        return;
    }

    let max_points = asset_meta.point_count;
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

    if !instance_data.is_empty() {
        commands.spawn((
            Mesh3d(
                meshes.add(
                    crate::engine::mesh::point_index_mesh::create_point_index_mesh(max_points),
                ),
            ),
            InstancedAssetData(instance_data),
            Transform::IDENTITY,
            NoFrustumCulling,
            bevy::render::view::NoIndirectDrawing,
            Name::new("InstancedAssetRenderer"),
        ));
    }
}
