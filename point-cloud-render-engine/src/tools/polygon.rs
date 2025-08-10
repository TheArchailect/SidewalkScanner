use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::primitives;
use bevy::window::PrimaryWindow;

use crate::engine::camera::ViewportCamera;
use crate::engine::grid::GroundGrid;
use crate::engine::point_cloud::PointCloudAssets;

#[derive(Component)]
pub struct PolygonPoints;

#[derive(Component)]
pub struct PolygonLines;

#[derive(Component)]
pub struct PolygonFill;

#[derive(Component)]
pub struct PolygonPreview;

#[derive(Component)]
pub struct CompletedPolygon {
    pub id: u32,
}

#[derive(Resource)]
pub struct PolygonTool {
    pub is_active: bool,
    pub current_polygon: Vec<Vec3>,
    pub preview_point: Option<Vec3>,
    pub is_completed: bool,
}

impl Default for PolygonTool {
    fn default() -> Self {
        Self {
            is_active: false,
            current_polygon: Vec::new(),
            preview_point: None,
            is_completed: false,
        }
    }
}

#[derive(Resource)]
pub struct PolygonCounter {
    pub next_id: u32,
}

impl Default for PolygonCounter {
    fn default() -> Self {
        Self { next_id: 0 }
    }
}

pub fn polygon_tool_system(
    mut commands: Commands,
    mut polygon_tool: ResMut<PolygonTool>,
    mut polygon_counter: ResMut<PolygonCounter>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    viewport_camera: Res<ViewportCamera>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
) {
    // Toggle polygon tool with 'P' key
    if keyboard.just_pressed(KeyCode::KeyP) {
        polygon_tool.is_active = !polygon_tool.is_active;
        if polygon_tool.is_active {
            println!(
                "Polygon tool activated. Left click to add points, Right click to complete, 'C' to clear current, 'X' to clear all."
            );
        } else {
            println!("Polygon tool deactivated.");
            polygon_tool.current_polygon.clear();
            polygon_tool.preview_point = None;
            polygon_tool.is_completed = false;
        }
    }

    if !polygon_tool.is_active {
        return;
    }

    // Clear current polygon with 'C' key
    if keyboard.just_pressed(KeyCode::KeyC) {
        polygon_tool.current_polygon.clear();
        polygon_tool.preview_point = None;
        polygon_tool.is_completed = false;
        println!("Current polygon cleared.");
    }

    // Clear all polygons with 'X' key
    if keyboard.just_pressed(KeyCode::KeyX) {
        polygon_tool.current_polygon.clear();
        polygon_tool.preview_point = None;
        polygon_tool.is_completed = false;
        println!("All polygons cleared.");
    }

    // Update preview point only if not completed
    if !polygon_tool.is_completed {
        if let (Ok((camera_global_transform, camera)), Ok(window)) =
            (camera_query.single(), windows.single())
        {
            if let Some(cursor_pos) = window.cursor_position() {
                polygon_tool.preview_point = viewport_camera.mouse_to_ground_plane(
                    cursor_pos,
                    camera,
                    camera_global_transform,
                    images.get(&assets.heightmap_texture),
                    assets.bounds.as_ref(),
                );
            }
        }
    }

    // Handle mouse clicks only if not completed
    if !polygon_tool.is_completed {
        if mouse_button.just_pressed(MouseButton::Left) {
            // Add point to polygon
            if let Some(point) = polygon_tool.preview_point {
                polygon_tool.current_polygon.push(point);
                println!(
                    "Added polygon point {} at ({:.2}, {:.2}, {:.2})",
                    polygon_tool.current_polygon.len(),
                    point.x,
                    point.y,
                    point.z
                );
            }
        }

        if mouse_button.just_pressed(MouseButton::Right) && polygon_tool.current_polygon.len() >= 3
        {
            // Complete the polygon
            polygon_tool.is_completed = true;
            polygon_tool.preview_point = None;

            // Create a completed polygon entity
            let polygon_id = polygon_counter.next_id;
            polygon_counter.next_id += 1;

            create_completed_polygon(
                &mut commands,
                &polygon_tool.current_polygon,
                polygon_id,
                viewport_camera.ground_height,
                &mut meshes,
                &mut materials,
            );

            println!(
                "Polygon {} completed with {} points",
                polygon_id,
                polygon_tool.current_polygon.len()
            );

            // Clear current polygon to start a new one
            polygon_tool.current_polygon.clear();
            polygon_tool.is_completed = false;
        }
    }
}

fn create_completed_polygon(
    commands: &mut Commands,
    points: &[Vec3],
    polygon_id: u32,
    ground_height: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    if points.len() < 3 {
        return;
    }

    // Create points for completed polygon
    for (i, point) in points.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                ..default()
            })),
            Transform::from_translation(*point),
            CompletedPolygon { id: polygon_id },
        ));
    }

    // Create lines for completed polygon (including closing line)
    for i in 0..points.len() {
        let start = points[i];
        let end = points[(i + 1) % points.len()];

        let direction = end - start;
        let distance = direction.length();
        let midpoint = (start + end) * 0.5;

        if distance > 0.1 {
            let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());

            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(distance, 0.25, 0.25))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::hsv(0., 1., 1.),
                    emissive: LinearRgba::new(1., 1., 1., 1.),
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rotation),
                CompletedPolygon { id: polygon_id },
            ));
        }
    }

    // Create fill for completed polygon
    let polygon_mesh = create_polygon_mesh(points, ground_height);

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.25))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::hsv(0., 1., 1.),
            emissive: LinearRgba::new(1., 1., 1., 1.),
            ..default()
        })),
        CompletedPolygon { id: polygon_id },
    ));
}

pub fn update_polygon_preview(
    mut commands: Commands,
    polygon_tool: Res<PolygonTool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_preview: Query<Entity, With<PolygonPreview>>,
) {
    // Clean up existing preview entities
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn();
    }

    if !polygon_tool.is_active || polygon_tool.is_completed {
        return;
    }

    // Create preview point if we have one
    if let Some(preview_point) = polygon_tool.preview_point {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                ..default()
            })),
            Transform::from_translation(preview_point),
            PolygonPreview,
        ));

        // If we have existing points, show preview line to current mouse position
        if let Some(last_point) = polygon_tool.current_polygon.last() {
            let direction = preview_point - *last_point;
            let distance = direction.length();
            let midpoint = (*last_point + preview_point) * 0.5;

            if distance > 0.1 {
                let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());

                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(distance, 0.15, 0.15))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::hsv(0., 1., 1.),
                        emissive: LinearRgba::new(1., 1., 1., 1.),
                        ..default()
                    })),
                    Transform::from_translation(midpoint).with_rotation(rotation),
                    PolygonPreview,
                ));
            }
        }
    }
}

pub fn update_polygon_render(
    mut commands: Commands,
    polygon_tool: Res<PolygonTool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    existing_points: Query<Entity, (With<PolygonPoints>, Without<GroundGrid>)>,
    existing_lines: Query<Entity, (With<PolygonLines>, Without<GroundGrid>)>,
    existing_fill: Query<Entity, (With<PolygonFill>, Without<GroundGrid>)>,
    completed_polygons: Query<Entity, With<CompletedPolygon>>,
) {
    // Clear all polygons if 'X' was pressed
    if keyboard.just_pressed(KeyCode::KeyX) {
        for entity in completed_polygons.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Only render current polygon if tool is active and not completed
    if !polygon_tool.is_active
        || polygon_tool.is_completed
        || polygon_tool.current_polygon.is_empty()
    {
        // Clean up current polygon entities when not active or completed
        for entity in existing_points
            .iter()
            .chain(existing_lines.iter())
            .chain(existing_fill.iter())
        {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Clean up existing entities to redraw current polygon
    for entity in existing_points
        .iter()
        .chain(existing_lines.iter())
        .chain(existing_fill.iter())
    {
        commands.entity(entity).despawn();
    }

    // Render current polygon points
    for (i, point) in polygon_tool.current_polygon.iter().enumerate() {
        let color = if i == 0 {
            Color::srgb(1., 0., 0.);
        } else {
            Color::srgb(0.5, 0., 0.);
        };

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                ..default()
            })),
            Transform::from_translation(*point),
            PolygonPoints,
        ));
    }

    // Render current polygon lines (but don't close the loop until completed)
    for i in 0..(polygon_tool.current_polygon.len() - 1) {
        let start = polygon_tool.current_polygon[i];
        let end = polygon_tool.current_polygon[i + 1];

        let direction = end - start;
        let distance = direction.length();
        let midpoint = (start + end) * 0.5;

        if distance > 0.1 {
            let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(distance, 0.2, 0.2))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::hsv(0., 1., 1.),
                    emissive: LinearRgba::new(1., 1., 1., 1.),
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rotation),
                PolygonLines,
            ));
        }
    }
}

fn create_polygon_mesh(points: &[Vec3], ground_height: f32) -> Mesh {
    if points.len() < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Add all polygon points at ground height
    for point in points {
        vertices.push([point.x, ground_height + 0.1, point.z]);
    }

    // Simple fan triangulation from first vertex
    for i in 1..(points.len() - 1) {
        indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    // Generate normals pointing up
    let normals: Vec<[f32; 3]> = (0..points.len()).map(|_| [0.0, 1.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
