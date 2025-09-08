use bevy::prelude::*;

use crate::engine::assets::bounds::PointCloudBounds;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::loading::progress::LoadingProgress;
use crate::engine::mesh::point_index_mesh::PointCloud;
use crate::engine::mesh::point_index_mesh::create_point_index_mesh;
use crate::engine::point_cloud_render_pipeline::PointCloudRenderable;
use crate::engine::scene::gizmos::create_direction_gizmo;
use crate::engine::scene::grid::GridCreated;
use crate::engine::scene::grid::create_ground_grid;

pub fn create_point_cloud_when_ready(
    mut loading_progress: ResMut<LoadingProgress>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
    images: ResMut<Assets<Image>>,
    mut grid_created: ResMut<GridCreated>,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<SceneManifest>>,
) {
    if loading_progress.point_cloud_created || !loading_progress.textures_configured {
        return;
    }

    let Some(bounds) = &assets.get_bounds(&manifests) else {
        return;
    };

    // Create point cloud entity without material - custom pipeline handles rendering.
    spawn_point_cloud_entity(&mut commands, &mut meshes, bounds);

    // Create grid with standard material pipeline.
    if !grid_created.created {
        let heightmap_image = images.get(&assets.heightmap_texture);
        create_ground_grid(
            &mut commands,
            bounds,
            &mut meshes,
            &mut standard_materials,
            heightmap_image,
        );
        grid_created.created = true;
        println!("Grid created");
    }

    // Create gizmos using standard material system.
    spawn_gizmos(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &asset_server,
    );

    assets.is_loaded = true;
    loading_progress.point_cloud_created = true;
    println!("Point cloud and visual elements ready");
}

fn spawn_point_cloud_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    bounds: &PointCloudBounds,
) {
    // Create indexed vertex buffer for GPU-side point expansion.
    let mesh = create_point_index_mesh(bounds.loaded_points);

    commands.spawn((
        // Standard 3D mesh component without material binding.
        Mesh3d(meshes.add(mesh)),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        GlobalTransform::default(),
        // Point cloud identification for systems and queries.
        PointCloud,
        // Custom render pipeline component containing vertex count for draw calls.
        PointCloudRenderable {
            point_count: bounds.loaded_points as u32,
        },
        // Disable frustum culling for large-scale point cloud rendering.
        bevy::render::view::NoFrustumCulling,
    ));

    println!(
        "Point cloud entity spawned with {} vertices using custom render pipeline",
        bounds.loaded_points
    );
}

fn spawn_gizmos(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    create_direction_gizmo(commands, meshes, materials, asset_server);
}
