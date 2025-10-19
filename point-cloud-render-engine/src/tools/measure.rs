use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use constants::render_settings::{
    DRAW_LINE_WIDTH, DRAW_VERTEX_SIZE, MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub id: u32,
    pub start: Vec3,
    pub end: Vec3,
    pub distance: f32,
}

#[derive(Resource, Default)]
pub struct MeasureTool {
    pub is_active: bool,
    pub start_point: Option<Vec3>,
    pub preview_point: Option<Vec3>,
    pub next_id: u32,
    pub current: Option<Measurement>,
}

impl MeasureTool {
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
        if !active {
            self.start_point = None;
            self.preview_point = None;
            self.current = None;
        }
    }
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[derive(Component)]
pub struct MeasurePreview;

#[derive(Component)]
pub struct CompletedMeasurementTag;

// Input/logic: click to start, move to preview, click to finish
// Starting new measurement deletes previous one
pub fn measure_tool_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut measure_tool: ResMut<MeasureTool>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    viewport_camera: Option<ResMut<ViewportCamera>>,
    images: Res<Assets<Image>>,
    assets: Res<PointCloudAssets>,
    manifests: Res<Assets<SceneManifest>>,
    mut rpc_interface: ResMut<crate::rpc::web_rpc::WebRpcInterface>,
    existing_preview: Query<Entity, With<MeasurePreview>>,
) {
    if !measure_tool.is_active() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(scene_bounds) = assets.get_bounds(&manifests) else {
        return;
    };
    let Some(height_img) = images.get(&assets.heightmap_texture) else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Some(mut viewport_camera) = viewport_camera else {
        return;
    };
    let Ok((cam_xform, camera)) = cameras.single() else {
        return;
    };

    // Raycast from mouse to ground plane
    let hit = viewport_camera.mouse_to_ground_plane(
        cursor_pos,
        camera,
        cam_xform,
        Some(height_img),
        &scene_bounds,
    );
    let Some(hit) = hit else {
        return;
    };

    // Clean up existing preview entities to prevent accumulation.
    for entity in existing_preview.iter() {
        commands.entity(entity).despawn();
    }

    // mouse ray case visualisation
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::hsv(0., 1., 1.),
            emissive: LinearRgba::new(1., 1., 1., 1.),
            depth_bias: 0.0,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(hit),
        MeasurePreview,
        RenderLayers::layer(1),
    ));

    // Update cursor projection each frame for live preview
    let Some(bounds) = assets.get_bounds(&manifests) else {
        return;
    };
    if let (Ok((cam_tf, cam)), Ok(window)) = (cameras.single(), windows.single()) {
        if let Some(cursor) = window.cursor_position() {
            measure_tool.preview_point = viewport_camera.mouse_to_ground_plane(
                cursor,
                cam,
                cam_tf,
                images.get(&assets.heightmap_texture),
                &bounds,
            );
        } else {
            measure_tool.preview_point = None;
        }
    }

    // FIRST CLICK: start and clear any existing completed measurement
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(preview) = measure_tool.preview_point {
            match measure_tool.start_point {
                None => {
                    // If a previous completed measurement exists, drop it and clear UI
                    if measure_tool.current.take().is_some() {
                        rpc_interface.send_notification("measure_clear", serde_json::json!({}));
                    }

                    // Begin a new measurement
                    measure_tool.start_point = Some(preview);
                    rpc_interface.send_notification(
                        "measure_started",
                        serde_json::json!({ "position": [preview.x, preview.y, preview.z] }),
                    );
                }
                Some(start) => {
                    // SECOND CLICK: finalise the measurement
                    let end = preview;
                    let dist = start.distance(end);
                    let m = Measurement {
                        id: measure_tool.next_id,
                        start,
                        end,
                        distance: dist,
                    };
                    measure_tool.next_id += 1;
                    measure_tool.current = Some(m.clone());

                    rpc_interface.send_notification(
                        "measure_completed",
                        serde_json::json!({
                            "id": m.id,
                            "start": [m.start.x, m.start.y, m.start.z],
                            "end": [m.end.x, m.end.y, m.end.z],
                            "distance": m.distance,
                        }),
                    );

                    // Reset for next line
                    measure_tool.start_point = None;
                    measure_tool.preview_point = None;
                }
            }
        }
    }

    // Live measurement update
    if let (Some(start), Some(preview)) = (measure_tool.start_point, measure_tool.preview_point) {
        let dist = start.distance(preview);
        rpc_interface.send_notification(
            "measure_updated",
            serde_json::json!({
                "start": [start.x, start.y, start.z],
                "end": [preview.x, preview.y, preview.z],
                "distance": dist,
            }),
        );
    }
}

// Renderer: clears previous meshes each frame and rebuilds from state
pub fn update_measure_render(
    mut commands: Commands,
    measure_tool: Res<MeasureTool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_preview: Query<Entity, With<MeasurePreview>>,
    existing_completed: Query<Entity, With<CompletedMeasurementTag>>,
) {
    // Clear previews every frame
    for e in &existing_preview {
        commands.entity(e).despawn();
    }
    // Clear completed visuals, rebuild from state
    for e in &existing_completed {
        commands.entity(e).despawn();
    }

    // Preview measurement before completion
    if let (Some(start), Some(preview)) = (measure_tool.start_point, measure_tool.preview_point) {
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 1.0, 0.2),
                emissive: LinearRgba::new(1., 1., 0.2, 1.),
                unlit: true,
                ..default()
            })),
            Transform::from_translation(preview),
            MeasurePreview,
            RenderLayers::layer(1),
        ));

        let dir = preview - start;
        let dist = dir.length();
        if dist > 0.02 {
            let midpoint = (start + preview) * 0.5;
            let rot = Quat::from_rotation_arc(Vec3::X, dir.normalize());
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(dist, DRAW_LINE_WIDTH, DRAW_LINE_WIDTH))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 1.0, 0.2),
                    emissive: LinearRgba::new(1., 1., 0.2, 1.),
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rot),
                MeasurePreview,
                RenderLayers::layer(1),
            ));
        }
    }

    // Completed measurement
    if let Some(m) = &measure_tool.current {
        let dir = m.end - m.start;
        let dist = dir.length();
        if dist > 0.02 {
            let midpoint = (m.start + m.end) * 0.5;
            let rot = Quat::from_rotation_arc(Vec3::X, dir.normalize());
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(dist, DRAW_LINE_WIDTH, DRAW_LINE_WIDTH))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.27, 0.0),
                    emissive: LinearRgba::new(1., 0.5, 0., 1.),
                    unlit: true,
                    ..default()
                })),
                Transform::from_translation(midpoint).with_rotation(rot),
                CompletedMeasurementTag,
                RenderLayers::layer(1),
            ));
        }
    }
}

pub struct MeasureToolPlugin;
impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureTool>()
            .add_systems(Update, (measure_tool_system, update_measure_render));
    }
}
