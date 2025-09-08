use bevy::prelude::*;
use bevy::{render::mesh::PrimitiveTopology, render::render_asset::RenderAssetUsages};
#[derive(Component)]
pub struct PointCloud;

/// Create point index mesh for GPU-side vertex expansion in custom pipeline.
/// Generates triangle-based geometry that expands to screen-aligned quads per point.
pub fn create_point_index_mesh(point_count: usize) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList, // Triangle-based point rendering.
        RenderAssetUsages::RENDER_WORLD,
    );

    // Generate 6 vertices per point (2 triangles forming screen-aligned quad).
    // Vertex shader uses vertex index to determine point and triangle position.
    let vertex_count = point_count * 6;
    let indices: Vec<[f32; 3]> = (0..vertex_count).map(|i| [i as f32, 0.0, 0.0]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, indices);
    mesh
}
