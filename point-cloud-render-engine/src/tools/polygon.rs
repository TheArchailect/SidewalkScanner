use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::scene::grid::GroundGrid;
use crate::engine::systems::render_mode::RenderMode;
use crate::engine::systems::render_mode::RenderModeState;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::mesh::PrimitiveTopology;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use serde::{Deserialize, Serialize};

/// Polygon operations events
#[derive(Event, Debug, Clone)]
pub struct PolygonHideRequestEvent{
    pub source_items: Vec<(String, String)>,    // category_id, item_id
}

#[derive(Event, Debug, Clone)]
pub struct PolygonReclassifyRequestEvent {
    pub source_items: Vec<(String, String)>,
    pub target: (String, String),               // target_category_id, target_item_id
}

pub struct PolygonToolPlugin;
impl Plugin for PolygonToolPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<PolygonHideRequestEvent>()
            .add_event::<PolygonReclassifyRequestEvent>()
            .add_systems(PostUpdate, process_poloygon_hide_requests)
            .add_systems(PostUpdate, process_polygon_reclassify_requests);
    }
}

/// Polygon system which applies Hide operation
fn process_poloygon_hide_requests(
    mut ev: EventReader<PolygonHideRequestEvent>,
    // TODO: add resources/queries needed to perform the actual hide - for geometry + data mutation
    // Examples (adjust to your project paths/types):
    // assets store to access/classify/mutate points
    // assets: ResMut<PointCloudAssets>,
    //
    // active polygon state (whatever you use to store the finished polygon loop)
    // polygon_state: Res<YourPolygonState>,
) {
    for e in ev.read() {
        // TODO: find points inside active polygon, filter by e.source_items, mark hidden, update counts
        // 1) Get the current/active polygon (world-space).
        //    Replace this with your real accessor.
        //    The usual convention is projecting XZ plane; if your ground plane is XY, adapt the projection below.
        // --------------------------------------------------------------------
        // let polygon_world_2d: Vec<Vec2> = polygon_state.vertices.iter()
        //     .map(|v3| Vec2::new(v3.x, v3.z)) // project to XZ
        //     .collect();
        // if polygon_world_2d.len() < 3 { continue; }

        // 2) Compute a conservative AABB around the polygon to cheaply cull candidates.
        // --------------------------------------------------------------------
        // let (min, max) = aabb_from_poly(&polygon_world_2d);

        // 3) Iterate candidate points in that AABB and do a precise point-in-polygon test.
        //    Filter by e.source_items if provided.
        //    For performance, prefer an index/histogram in your store rather than scanning every point.
        // --------------------------------------------------------------------
        // let mut affected: u64 = 0;
        // assets.for_each_point_in_aabb(min, max, |pid, p| {
        //     let p2 = Vec2::new(p.x, p.z); // project onto XZ
        //     if point_in_polygon_2d(p2, &polygon_world_2d) {
        //         if matches_source_items(&assets, pid, &e.source_items) {
        //             // mark hidden; replace with your real mutation call:
        //             assets.set_point_hidden(pid, true);
        //             affected += 1;
        //         }
        //     }
        // });
        info!("[POLY] hide request received; {} filter pairs", e.source_items.len());
    }
}

fn process_polygon_reclassify_requests(
    mut ev: EventReader<PolygonReclassifyRequestEvent>,

) {
    for e in ev.read() {
        let (target_Cat, target_Item) = &e.target;
        info!("[POLY] Reclassify request received; {} filter pairs -> target=({},{})", e.source_items.len(), target_Cat, target_Item)
    }
}

/// Polygon definition for point cloud classification operations.
/// Contains spatial coordinates and target classification metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationPolygon {
    pub id: u32,
    pub points: Vec<Vec3>, // XZ plane coordinates for heightmap intersection.
    pub new_class: u32,    // Target classification ID for enclosed points.
}

/// Resource containing active polygon classification data.
/// Extracted to render world for compute shader access.
#[derive(Resource, ExtractResource, Clone)]
pub struct PolygonClassificationData {
    pub polygons: Vec<ClassificationPolygon>,
    pub max_polygons: usize, // GPU uniform buffer size constraint.
}

impl Default for PolygonClassificationData {
    fn default() -> Self {
        Self {
            polygons: Vec::new(),
            max_polygons: 64, // Conservative limit for GPU uniform storage.
        }
    }
}

impl PolygonTool {
    /// Set the active state of the polygon tool.
    /// Called by the tool manager during activation and deactivation.
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;

        if !active {
            // Clear any in-progress polygon when tool is deactivated.
            self.current_polygon.clear();
        }
    }

    /// Check if the polygon tool is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

/// Component markers for polygon visualization entities.
/// Enables selective cleanup and rendering control.
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

/// Uniform data structure for polygon classification compute shaders.
/// Maintains compatibility with existing compute pipeline while removing
/// dependency on Bevy's Material trait system.
#[derive(Debug, Clone, Copy, bevy::render::render_resource::ShaderType)]
#[repr(C)]
pub struct PolygonClassificationUniform {
    pub polygon_count: u32,
    pub total_points: u32,
    pub render_mode: u32,
    pub _padding: u32,
    pub point_data: [Vec4; 512],
    pub polygon_info: [Vec4; 64],
}

impl Default for PolygonClassificationUniform {
    fn default() -> Self {
        Self {
            polygon_count: 0,
            total_points: 0,
            render_mode: RenderMode::RgbColour as u32,
            _padding: 0,
            point_data: [Vec4::ZERO; 512],
            polygon_info: [Vec4::ZERO; 64],
        }
    }
}

/// Interactive polygon creation tool state.
/// Manages user input, preview visualization, and completion logic.
#[derive(Resource)]
pub struct PolygonTool {
    pub is_active: bool,
    pub current_polygon: Vec<Vec3>,
    pub preview_point: Option<Vec3>,
    pub is_completed: bool,
    pub current_class: u32,
    pub target_point_spacing: f32, // Uniform resampling distance.
}

impl Default for PolygonTool {
    fn default() -> Self {
        Self {
            is_active: false,
            current_polygon: Vec::new(),
            preview_point: None,
            is_completed: false,
            current_class: 1,
            target_point_spacing: 1.0,
        }
    }
}

/// Resamples polygon edges to ensure uniform point distribution.
/// Prevents GPU compute shader performance issues from irregular spacing. specifically for our z-order morton codes we need we're looking up near points for each point of the polygons.
/// there for large polygons or triangles with no internal points will miss 'close' regions in the intersection collision checks
fn resample_polygon_uniform(points: &[Vec3], target_spacing: f32) -> Vec<Vec3> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut resampled = Vec::new();

    for i in 0..points.len() {
        let start = points[i];
        let end = points[(i + 1) % points.len()];

        // Calculate edge subdivision requirements for uniform spacing.
        let edge_length = (end - start).length();
        if edge_length < 0.001 {
            continue; // Skip degenerate edges to prevent numerical instability.
        }

        let num_segments = (edge_length / target_spacing).max(1.0) as usize;
        let actual_spacing = edge_length / num_segments as f32;

        // Generate uniformly distributed points along edge, excluding endpoints.
        // Prevents duplicate vertices at polygon corner intersections.
        for j in 0..num_segments {
            let t = j as f32 * actual_spacing / edge_length;
            let interpolated_point = start + (end - start) * t;
            resampled.push(interpolated_point);
        }
    }

    resampled
}

/// Unique identifier generator for polygon classification entities.
/// Enables deterministic polygon tracking across render and compute systems.
#[derive(Resource)]
pub struct PolygonCounter {
    pub next_id: u32,
}

impl Default for PolygonCounter {
    fn default() -> Self {
        Self { next_id: 0 }
    }
}

/// Primary polygon creation and interaction system.
/// Handles user input, ground intersection calculation, and polygon completion.
/// Now integrates with tool manager for activation state control and RPC completion.
pub fn polygon_tool_system(
    mut commands: Commands,
    mut polygon_tool: ResMut<PolygonTool>,
    mut polygon_counter: ResMut<PolygonCounter>,
    mut classification_data: ResMut<PolygonClassificationData>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut viewport_camera: ResMut<ViewportCamera>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<PointCloudAssets>,
    images: Res<Assets<Image>>,
    mut rpc_interface: ResMut<crate::rpc::web_rpc::WebRpcInterface>,
    manifests: Res<Assets<SceneManifest>>,
) {
    // Only process input when polygon tool is active (controlled by tool manager).
    if !polygon_tool.is_active {
        return;
    }

    // Map number keys to classification IDs for user convenience.
    for (key, class_id) in [
        (KeyCode::Digit1, 1),
        (KeyCode::Digit2, 2),
        (KeyCode::Digit3, 3),
        (KeyCode::Digit4, 4),
        (KeyCode::Digit5, 5),
        (KeyCode::Digit6, 6),
        (KeyCode::Digit7, 7),
        (KeyCode::Digit8, 8),
        (KeyCode::Digit9, 9),
    ] {
        if keyboard.just_pressed(key) {
            polygon_tool.current_class = class_id;
            println!("Classification class set to: {}", class_id);

            // Notify frontend of class change
            rpc_interface.send_notification(
                "polygon_class_changed",
                serde_json::json!({
                    "current_class": class_id
                }),
            );
        }
    }

    // Clear current polygon construction with 'O' key.
    if keyboard.just_pressed(KeyCode::KeyO) {
        polygon_tool.current_polygon.clear();
        polygon_tool.preview_point = None;
        polygon_tool.is_completed = false;
        println!("Current polygon cleared.");

        rpc_interface.send_notification(
            "polygon_cleared",
            serde_json::json!({
                "action": "clear_current"
            }),
        );
    }

    // Clear all completed polygons with 'I' key.
    if keyboard.just_pressed(KeyCode::KeyI) {
        polygon_tool.current_polygon.clear();
        polygon_tool.preview_point = None;
        polygon_tool.is_completed = false;
        classification_data.polygons.clear();
        println!("All polygons cleared.");

        rpc_interface.send_notification(
            "polygon_cleared",
            serde_json::json!({
                "action": "clear_all"
            }),
        );
    }

    let Some(bounds) = assets.get_bounds(&manifests) else {
        return;
    };

    // Update preview point for real-time cursor tracking.
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
                    &bounds,
                );
            }
        }
    }

    // Handle mouse click input for vertex addition.
    if !polygon_tool.is_completed {
        if mouse_button.just_pressed(MouseButton::Left) {
            if let Some(point) = polygon_tool.preview_point {
                polygon_tool.current_polygon.push(point);
                println!(
                    "Added polygon point {} at ({:.2}, {:.2}, {:.2})",
                    polygon_tool.current_polygon.len(),
                    point.x,
                    point.y,
                    point.z
                );

                // Notify frontend of point addition
                rpc_interface.send_notification(
                    "polygon_point_added",
                    serde_json::json!({
                        "point_count": polygon_tool.current_polygon.len(),
                        "position": [point.x, point.y, point.z]
                    }),
                );
            }
        }

        // Complete polygon construction with Shift key OR via RPC completion flag
        let should_complete = (keyboard.just_pressed(KeyCode::ShiftLeft)
            || polygon_tool.is_completed)
            && polygon_tool.current_polygon.len() >= 3;

        if should_complete {
            let resampled_points = resample_polygon_uniform(
                &polygon_tool.current_polygon,
                polygon_tool.target_point_spacing,
            );

            // Generate unique polygon identifier for tracking.
            let polygon_id = polygon_counter.next_id;
            polygon_counter.next_id += 1;

            // Create classification data structure for compute shader processing.
            let class_polygon = ClassificationPolygon {
                id: polygon_id,
                points: resampled_points.clone(),
                new_class: polygon_tool.current_class,
            };

            // Add to classification data with GPU memory constraint validation.
            if classification_data.polygons.len() < classification_data.max_polygons {
                classification_data.polygons.push(class_polygon);
                println!(
                    "Added classification polygon {} with class {}",
                    polygon_id, polygon_tool.current_class
                );

                // Create visual representation using standard material pipeline.
                create_completed_polygon(
                    &mut commands,
                    &resampled_points,
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

                // Notify frontend of successful completion
                rpc_interface.send_notification(
                    "polygon_completed",
                    serde_json::json!({
                        "polygon_id": polygon_id,
                        "point_count": polygon_tool.current_polygon.len(),
                        "class": polygon_tool.current_class,
                        "total_polygons": classification_data.polygons.len()
                    }),
                );
            } else {
                println!(
                    "Warning: Maximum polygon limit reached ({})",
                    classification_data.max_polygons
                );

                // Notify frontend of error
                rpc_interface.send_notification(
                    "polygon_error",
                    serde_json::json!({
                        "error": "Maximum polygon limit reached",
                        "max_polygons": classification_data.max_polygons
                    }),
                );
            }

            // Reset state for next polygon creation.
            polygon_tool.current_polygon.clear();
            polygon_tool.preview_point = None;
            polygon_tool.is_completed = false;
        }
    }
}
/// Updates polygon classification data for compute shader consumption.
/// Removed material system dependencies in favor of resource extraction.
pub fn update_polygon_classification_shader(
    classification_data: Res<PolygonClassificationData>,
    render_state: Res<RenderModeState>,
) {
    // Early exit if no polygon data changes detected.
    if !classification_data.is_changed() && !render_state.is_changed() {
        return;
    }

    // Create uniform data structure for GPU compute shader compatibility.
    // Data will be extracted to render world through resource system.
    let mut uniform = PolygonClassificationUniform::default();
    uniform.polygon_count = classification_data.polygons.len().min(64) as u32;
    uniform.render_mode = render_state.current_mode as u32;

    let mut point_offset = 0;
    for (i, polygon) in classification_data.polygons.iter().take(64).enumerate() {
        // Store polygon metadata: point offset, count, and classification ID.
        uniform.polygon_info[i] = Vec4::new(
            point_offset as f32,
            polygon.points.len() as f32,
            polygon.new_class as f32,
            0.0,
        );

        // Flatten polygon vertices into linear array for GPU processing.
        for point in &polygon.points {
            if point_offset < 512 {
                uniform.point_data[point_offset] = Vec4::new(point.x, point.z, 0.0, 0.0);
                point_offset += 1;
            }
        }
    }

    uniform.total_points = point_offset as u32;

    // Note: Uniform data stored in classification_data resource.
    // Compute classification system extracts this through resource system
    // rather than through deprecated material uniform updates.
}

/// Creates persistent visualization entities for completed polygons.
/// Uses standard material pipeline for UI elements separate from point cloud.
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

    // Create vertex markers with emissive material for visibility.
    for (i, point) in points.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                depth_bias: 0.0,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(*point),
            CompletedPolygon { id: polygon_id },
            RenderLayers::layer(1),
        ));
    }

    // Create edge visualization with oriented cuboid meshes.
    for i in 0..points.len() {
        let start = points[i];
        let end = points[(i + 1) % points.len()]; // Close polygon loop.

        let direction = end - start;
        let distance = direction.length();
        let midpoint = (start + end) * 0.5;

        if distance > 0.1 {
            // Calculate edge orientation for proper cuboid alignment.
            let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());

            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(distance, 0.025, 0.025))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::hsv(0., 1., 1.),
                    emissive: LinearRgba::new(1., 1., 1., 1.),
                    depth_bias: 0.0,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rotation),
                CompletedPolygon { id: polygon_id },
                RenderLayers::layer(1),
            ));
        }
    }

    // Create polygon fill mesh for visual feedback (optional).
    let _fill_mesh = create_polygon_mesh(points, ground_height);

    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.05))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::hsv(0., 1., 1.),
            emissive: LinearRgba::new(1., 1., 1., 1.),
            depth_bias: 0.0,
            unlit: true,
            ..default()
        })),
        CompletedPolygon { id: polygon_id },
        RenderLayers::layer(1),
    ));
}

/// Manages real-time preview visualization during polygon construction.
/// Shows current cursor position and preview edge to last vertex.
pub fn update_polygon_preview(
    mut commands: Commands,
    polygon_tool: Res<PolygonTool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_preview: Query<Entity, With<PolygonPreview>>,
) {
    // Clean up existing preview entities to prevent accumulation.
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn();
    }

    if !polygon_tool.is_active || polygon_tool.is_completed {
        return;
    }

    // Create preview cursor visualization at mouse intersection point.
    if let Some(preview_point) = polygon_tool.preview_point {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.05))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 1., 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                depth_bias: 0.0,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(preview_point),
            PolygonPreview,
            RenderLayers::layer(1),
        ));

        // Show preview edge from last placed vertex to current cursor position.
        if let Some(last_point) = polygon_tool.current_polygon.last() {
            let direction = preview_point - *last_point;
            let distance = direction.length();
            let midpoint = (*last_point + preview_point) * 0.5;

            if distance > 0.1 {
                let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());

                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(distance, 0.025, 0.025))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::hsv(0., 1., 1.),
                        emissive: LinearRgba::new(1., 1., 1., 1.),
                        depth_bias: 0.0,
                        unlit: true,
                        ..default()
                    })),
                    Transform::from_translation(midpoint).with_rotation(rotation),
                    PolygonPreview,
                    RenderLayers::layer(1),
                ));
            }
        }
    }
}

/// Updates current polygon visualization during interactive construction.
/// Manages vertex markers and edge visualization for incomplete polygons.
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
    // Handle bulk polygon deletion with 'X' key input.
    if keyboard.just_pressed(KeyCode::KeyX) {
        for entity in completed_polygons.iter() {
            commands.entity(entity).despawn();
        }
    }

    // Clean up current polygon visualization when tool inactive.
    if !polygon_tool.is_active
        || polygon_tool.is_completed
        || polygon_tool.current_polygon.is_empty()
    {
        for entity in existing_points
            .iter()
            .chain(existing_lines.iter())
            .chain(existing_fill.iter())
        {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Clean up existing entities for fresh rendering of current state.
    for entity in existing_points
        .iter()
        .chain(existing_lines.iter())
        .chain(existing_fill.iter())
    {
        commands.entity(entity).despawn();
    }

    // Render vertex markers for current polygon construction.
    for (_i, point) in polygon_tool.current_polygon.iter().enumerate() {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.1))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::hsv(0., 0.5, 1.),
                emissive: LinearRgba::new(1., 1., 1., 1.),
                depth_bias: 0.0,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(*point),
            PolygonPoints,
            RenderLayers::layer(1),
        ));
    }

    // Render edges between consecutive vertices (open polygon).
    // Intentionally excludes closing edge until polygon completion.
    for i in 0..(polygon_tool.current_polygon.len() - 1) {
        let start = polygon_tool.current_polygon[i];
        let end = polygon_tool.current_polygon[i + 1];

        let direction = end - start;
        let distance = direction.length();
        let midpoint = (start + end) * 0.5;

        if distance > 0.1 {
            let rotation = Quat::from_rotation_arc(Vec3::X, direction.normalize());
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(distance, 0.045, 0.045))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::hsv(0., 1., 1.),
                    emissive: LinearRgba::new(1., 1., 1., 1.),
                    depth_bias: 0.0,
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rotation),
                PolygonLines,
                RenderLayers::layer(1),
            ));
        }
    }
}

/// Creates triangulated mesh representation of polygon for fill visualization.
/// Uses simple fan triangulation suitable for convex and simple concave shapes.
fn create_polygon_mesh(points: &[Vec3], ground_height: f32) -> Mesh {
    if points.len() < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Project all vertices to consistent ground plane for proper triangulation.
    for point in points {
        vertices.push([point.x, ground_height + 0.1, point.z]);
    }

    // Fan triangulation from first vertex to create triangle list.
    for i in 1..(points.len() - 1) {
        indices.extend_from_slice(&[0, i as u32, (i + 1) as u32]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));

    // Generate upward-facing normals for consistent lighting behavior.
    let normals: Vec<[f32; 3]> = (0..points.len()).map(|_| [0.0, 1.0, 0.0]).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    mesh
}
