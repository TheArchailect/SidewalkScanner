use bevy::{prelude::*, render::mesh::PrimitiveTopology};

pub fn create_point_index_mesh(point_count: usize) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::PointList);
    let indices: Vec<[f32; 3]> = (0..point_count).map(|i| [i as f32, 0.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
    mesh
}

pub fn create_heightfield_line_mesh(
    bounds: &super::point_cloud::PointCloudBounds,
    heightmap_image: &Image,
    fixed_coord: f32,
    is_x_direction: bool,
    start_coord: f32,
    end_coord: f32,
    segments: usize,
) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let step = (end_coord - start_coord) / segments as f32;

    for i in 0..=segments {
        let varying_coord = start_coord + i as f32 * step;

        let (world_x, world_z) = if is_x_direction {
            (fixed_coord, varying_coord)
        } else {
            (varying_coord, fixed_coord)
        };

        let norm_x = (world_x - bounds.min_x as f32) / (bounds.max_x - bounds.min_x) as f32;
        let norm_z = (world_z - bounds.min_z as f32) / (bounds.max_z - bounds.min_z) as f32;

        let height = if norm_x >= 0.0 && norm_x <= 1.0 && norm_z >= 0.0 && norm_z <= 1.0 {
            super::point_cloud::sample_heightmap(heightmap_image, norm_x, norm_z, bounds) + 0.5
        } else {
            bounds.min_y as f32 + 0.5
        };

        vertices.push([world_x, height, world_z]);
    }

    for i in 0..segments {
        indices.extend_from_slice(&[i as u32, (i + 1) as u32]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}
