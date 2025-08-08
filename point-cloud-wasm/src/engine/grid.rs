use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::view::NoFrustumCulling;

use super::point_cloud::PointCloudBounds;
use super::point_cloud::sample_heightmap;

#[derive(Component)]
pub struct GroundGrid;

#[derive(Resource, Default)]
pub struct GridCreated {
    pub created: bool,
}

pub fn create_ground_grid(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    heightmap_image: Option<&Image>,
) {
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.5, 0.5, 0.5, 0.3),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    if let Some(heightmap) = heightmap_image {
        create_heightfield_grid(
            commands,
            bounds,
            meshes,
            materials,
            heightmap,
            grid_material,
        );
    } else {
        eprintln!("Error: Heightmap expected but not provided!");
    }
}

fn create_heightfield_grid(
    commands: &mut Commands,
    bounds: &PointCloudBounds,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    heightmap_image: &Image,
    grid_material: Handle<StandardMaterial>,
) {
    let size = bounds.size();
    let grid_size_x = size.x;
    let grid_size_z = size.z;

    // Target cell size in meters (adjust this for desired grid density)
    let target_cell_size = 1.0; // 1 meter grid cells

    // Calculate optimal line counts based on target cell size
    let line_count_x = (grid_size_x / target_cell_size).round() as u32;
    let line_count_z = (grid_size_z / target_cell_size).round() as u32;

    // Ensure minimum grid resolution
    let line_count_x = line_count_x.max(10);
    let line_count_z = line_count_z.max(10);

    // Calculate actual cell sizes (will be close to target)
    let actual_cell_size_x = grid_size_x / line_count_x as f32;
    let actual_cell_size_z = grid_size_z / line_count_z as f32;

    // Segments per line based on cell size for consistent detail
    let segments_per_line = (target_cell_size * 20.0) as usize; // 20 segments per meter
    let segments_per_line = segments_per_line.clamp(50, 1000); // Reasonable bounds

    println!("Grid info:");
    println!("  Data size: {:.1}m x {:.1}m", grid_size_x, grid_size_z);
    println!("  Target cell size: {:.1}m", target_cell_size);
    println!("  Grid resolution: {}x{} lines", line_count_x, line_count_z);
    println!(
        "  Actual cell size: {:.2}m x {:.2}m",
        actual_cell_size_x, actual_cell_size_z
    );
    println!("  Segments per line: {}", segments_per_line);
    println!(
        "  Total entities: {} lines",
        (line_count_x + 1) + (line_count_z + 1)
    );

    // Create X-direction lines (running along Z axis, fixed X positions)
    let line_spacing_x = grid_size_x / line_count_x as f32;
    for i in 0..=line_count_x {
        let x_offset = bounds.min_x as f32 + i as f32 * line_spacing_x;

        let line_mesh = create_heightfield_line_mesh(
            bounds,
            heightmap_image,
            x_offset,
            true, // is_x_direction (line runs along Z, X is fixed)
            bounds.min_z as f32,
            bounds.max_z as f32,
            segments_per_line,
        );

        commands.spawn((
            Mesh3d(meshes.add(line_mesh)),
            MeshMaterial3d(grid_material.clone()),
            Visibility::Visible,
            NoFrustumCulling,
            Transform::IDENTITY,
            GroundGrid,
        ));
    }

    // Create Z-direction lines (running along X axis, fixed Z positions)
    let line_spacing_z = grid_size_z / line_count_z as f32;
    for i in 0..=line_count_z {
        let z_offset = bounds.min_z as f32 + i as f32 * line_spacing_z;

        let line_mesh = create_heightfield_line_mesh(
            bounds,
            heightmap_image,
            z_offset,
            false, // is_x_direction (line runs along X, Z is fixed)
            bounds.min_x as f32,
            bounds.max_x as f32,
            segments_per_line,
        );

        commands.spawn((
            Mesh3d(meshes.add(line_mesh)),
            MeshMaterial3d(grid_material.clone()),
            Transform::IDENTITY,
            GroundGrid,
        ));
    }
}

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
            // Line runs along Z axis, X is fixed
            (fixed_coord, varying_coord)
        } else {
            // Line runs along X axis, Z is fixed
            (varying_coord, fixed_coord)
        };

        // Convert to normalized heightmap coordinates
        let norm_x = (world_x - bounds.min_x as f32) / (bounds.max_x - bounds.min_x) as f32;
        let norm_z = (world_z - bounds.min_z as f32) / (bounds.max_z - bounds.min_z) as f32;

        let height = if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
            sample_heightmap(heightmap_image, norm_x, norm_z, bounds) + 0.5 // Slightly above terrain
        } else {
            bounds.min_y as f32 + 0.5 // Default height outside bounds
        };

        vertices.push([world_x, height, world_z]);
    }

    // Create line segments
    for i in 0..segments {
        indices.extend_from_slice(&[i as u32, (i + 1) as u32]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::MAIN_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    mesh
}
