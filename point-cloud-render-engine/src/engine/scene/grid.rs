/// Heightfield-aware grid rendering with unified texture support
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::view::NoFrustumCulling;

use super::heightmap::sample_heightmap_bilinear;
use crate::engine::assets::bounds::PointCloudBounds;

#[derive(Component)]
pub struct GroundGrid;

#[derive(Resource, Default)]
pub struct GridCreated {
    pub created: bool,
}

/// Create ground grid following heightmap terrain
pub fn create_ground_grid(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    heightmap_image: Option<&Image>,
) {
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    if let Some(heightmap) = heightmap_image {
        create_heightfield_grid(commands, bounds, meshes, heightmap, grid_material);
    } else {
        eprintln!("Warning: Heightmap expected but not provided for grid generation");
    }
}

/// Generate heightfield-aware grid lines
fn create_heightfield_grid(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    heightmap_image: &Image,
    grid_material: Handle<StandardMaterial>,
) {
    let size = bounds.size();
    let grid_size_x = size.x;
    let grid_size_z = size.z;

    // Target cell size in metres
    let target_cell_size = 1.0;

    // Calculate optimal line counts based on target cell size
    let line_count_x = (grid_size_x / target_cell_size).round() as u32;
    let line_count_z = (grid_size_z / target_cell_size).round() as u32;

    // Ensure minimum grid resolution
    let line_count_x = line_count_x.max(10);
    let line_count_z = line_count_z.max(10);

    // Segments per line for smooth heightfield following
    let segments_per_line = (target_cell_size * 20.0) as usize;
    let segments_per_line = segments_per_line.clamp(50, 1000);

    create_grid_lines_x_direction(
        commands,
        bounds,
        meshes,
        heightmap_image,
        grid_material.clone(),
        line_count_x,
        grid_size_x,
        segments_per_line,
    );

    create_grid_lines_z_direction(
        commands,
        bounds,
        meshes,
        heightmap_image,
        grid_material,
        line_count_z,
        grid_size_z,
        segments_per_line,
    );
}

/// Create X-direction grid lines (running along Z axis, fixed X positions)
fn create_grid_lines_x_direction(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    heightmap_image: &Image,
    grid_material: Handle<StandardMaterial>,
    line_count_x: u32,
    grid_size_x: f32,
    segments_per_line: usize,
) {
    let line_spacing_x = grid_size_x / line_count_x as f32;

    for i in 0..=line_count_x {
        let x_offset = bounds.bounds.min_x as f32 + i as f32 * line_spacing_x;

        let line_mesh = create_heightfield_line_mesh(
            bounds,
            heightmap_image,
            x_offset,
            true, // is_x_direction
            bounds.bounds.min_z as f32,
            bounds.bounds.max_z as f32,
            segments_per_line,
        );

        spawn_grid_line_entity(commands, meshes, grid_material.clone(), line_mesh);
    }
}

/// Create Z-direction grid lines (running along X axis, fixed Z positions)
fn create_grid_lines_z_direction(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    heightmap_image: &Image,
    grid_material: Handle<StandardMaterial>,
    line_count_z: u32,
    grid_size_z: f32,
    segments_per_line: usize,
) {
    let line_spacing_z = grid_size_z / line_count_z as f32;

    for i in 0..=line_count_z {
        let z_offset = bounds.bounds.min_z as f32 + i as f32 * line_spacing_z;

        let line_mesh = create_heightfield_line_mesh(
            bounds,
            heightmap_image,
            z_offset,
            false, // is_x_direction
            bounds.bounds.min_x as f32,
            bounds.bounds.max_x as f32,
            segments_per_line,
        );

        spawn_grid_line_entity(commands, meshes, grid_material.clone(), line_mesh);
    }
}

/// Create single heightfield-following line mesh
fn create_heightfield_line_mesh(
    bounds: &PointCloudBounds,
    heightmap_image: &Image,
    fixed_coord: f32,     // The X or Z coordinate that stays constant
    is_x_direction: bool, // true if line runs along Z axis (X fixed), false if along X axis (Z fixed)
    start_coord: f32,     // Start of the varying coordinate
    end_coord: f32,       // End of the varying coordinate
    segments: usize,
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = (end_coord - start_coord) / segments as f32;

    // Generate vertices along the line, sampling heightmap
    for i in 0..=segments {
        let varying_coord = start_coord + i as f32 * step;

        let (world_x, world_z) = if is_x_direction {
            (fixed_coord, varying_coord)
        } else {
            (varying_coord, fixed_coord)
        };

        // Convert to normalised heightmap coordinates
        let norm_x = (world_x - bounds.bounds.min_x as f32)
            / (bounds.bounds.max_x - bounds.bounds.min_x) as f32;
        let norm_z = (world_z - bounds.bounds.min_z as f32)
            / (bounds.bounds.max_z - bounds.bounds.min_z) as f32;

        let height = if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
            sample_heightmap_bilinear(heightmap_image, norm_x, norm_z, bounds)
        } else {
            bounds.bounds.min_y as f32
        };

        vertices.push([world_x, height, world_z]);
    }

    // Create line segments
    for i in 0..segments {
        indices.extend_from_slice(&[i as u32, (i + 1) as u32]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}

/// Spawn grid line entity with no frustum culling
fn spawn_grid_line_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    grid_material: Handle<StandardMaterial>,
    line_mesh: Mesh,
) {
    commands.spawn((
        Mesh3d(meshes.add(line_mesh)),
        MeshMaterial3d(grid_material),
        Visibility::Visible,
        NoFrustumCulling,
        Transform::IDENTITY,
        GroundGrid,
    ));
}
