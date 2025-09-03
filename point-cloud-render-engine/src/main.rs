use crate::engine::compute_classification::{
    ComputeClassificationState, run_classification_compute,
};
use crate::engine::edl_compute_depth::{EDLRenderState, run_edl_compute};
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;
use engine::edl_compute_depth::EDLComputePlugin;
use tools::class_selection::{ClassSelectionState, handle_class_selection};
mod constants;
mod engine;
mod tools;
use crate::engine::edl_post_processing::{EDLPostProcessPlugin, EDLSettings};
use crate::engine::point_cloud::update_camera_uniform;
use engine::{
    camera::{ViewportCamera, camera_controller},
    gizmos::{create_direction_gizmo, update_direction_gizmo, update_mouse_intersection_gizmo},
    grid::GridCreated,
    point_cloud::point_cloud_asset_create,
    point_cloud::{PointCloudAssets, PointCloudBounds},
};
use tools::polygon::{
    PolygonClassificationData, PolygonCounter, PolygonTool, polygon_tool_system,
    update_polygon_classification_shader, update_polygon_preview, update_polygon_render,
};

use crate::engine::shaders::PointCloudShader;
use engine::compute_classification::ComputeClassificationPlugin;
use engine::render_mode::{RenderModeState, render_mode_system};

const RELATIVE_ASSET_PATH: &'static str = "pre_processor_data/riga_versions/riga_numbered_0.03";
const TEXTURE_RESOLUTION: &'static str = "2048x2048";

#[derive(Resource, Default)]
struct BoundsLoader {
    handle: Option<Handle<PointCloudBounds>>,
    loaded: bool,
}

fn main() {
    let mut app = create_app();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            app.run();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.run();
    }
}

/// Create application with unified texture pipeline support
fn create_app() -> App {
    let mut app = App::new();

    app.add_plugins(create_default_plugins())
        .add_plugins(MaterialPlugin::<PointCloudShader>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(JsonAssetPlugin::<PointCloudBounds>::new(&["json"]))
        .add_plugins(ComputeClassificationPlugin)
        .add_plugins(EDLComputePlugin)
        .add_plugins(EDLPostProcessPlugin);

    if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
        render_app
            .init_resource::<ComputeClassificationState>()
            .init_resource::<PolygonClassificationData>()
            .init_resource::<PointCloudAssets>()
            .init_resource::<RenderModeState>()
            .init_resource::<ClassSelectionState>()
            .init_resource::<EDLRenderState>()
            .init_resource::<SelectionBuffer>()
            .add_systems(
                bevy::render::Render,
                (run_classification_compute, run_edl_compute)
                    .chain()
                    .in_set(bevy::render::RenderSet::Queue),
            );
    }

    app.init_resource::<BoundsLoader>()
        .init_resource::<ClassSelectionState>()
        .init_resource::<SelectionBuffer>()
        .init_resource::<PolygonClassificationData>()
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<RenderModeState>()
        .init_resource::<GridCreated>()
        .insert_resource(create_point_cloud_assets(None))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                load_bounds_system,
                point_cloud_asset_create,
                handle_class_selection,
                fps_text_update_system,
                camera_controller,
                update_camera_uniform,
                update_direction_gizmo,
                update_mouse_intersection_gizmo,
                polygon_tool_system,
                update_polygon_preview,
                update_polygon_render,
                render_mode_system,
                update_selection_buffer,
                update_polygon_classification_shader,
            ),
        );

    app
}

/// Load bounds JSON and initialise camera
fn load_bounds_system(
    mut bounds_loader: ResMut<BoundsLoader>,
    mut assets: ResMut<PointCloudAssets>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bounds_assets: Res<Assets<PointCloudBounds>>,
) {
    // Start loading if not already started
    if bounds_loader.handle.is_none() {
        let bounds_path = get_bounds_path();
        println!("Loading bounds from: {}", bounds_path);
        bounds_loader.handle = Some(asset_server.load(&bounds_path));
        return;
    }

    // Check if loaded and not yet processed
    if !bounds_loader.loaded {
        if let Some(ref handle) = bounds_loader.handle {
            if let Some(bounds) = bounds_assets.get(handle) {
                println!("Successfully loaded bounds JSON");
                assets.bounds = Some(bounds.clone());
                bounds_loader.loaded = true;

                // Update camera with bounds
                if let Some(ref bounds) = assets.bounds {
                    let vp_camera = ViewportCamera::with_bounds(bounds);
                    commands.insert_resource(vp_camera);
                }
            }
        }
    }
}

/// Generate bounds file path for unified texture format
fn get_bounds_path() -> String {
    format!(
        "{}_metadata_{}.json",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    )
}

fn create_default_plugins() -> impl PluginGroup {
    let window_config = WindowPlugin {
        primary_window: Some(create_window_config()),
        ..default()
    };

    let asset_config = AssetPlugin {
        //file_path: "renderer/assets".into(),
        meta_check: AssetMetaCheck::Never,
        ..default()
    };

    DefaultPlugins.set(window_config).set(asset_config)
}

fn create_window_config() -> Window {
    #[cfg(target_arch = "wasm32")]
    {
        Window {
            canvas: Some("#bevy".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            present_mode: PresentMode::AutoVsync,
            ..default()
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        Window {
            present_mode: PresentMode::AutoVsync,
            ..default()
        }
    }
}

/// Create point cloud assets resource with unified texture format
fn create_point_cloud_assets(bounds: Option<PointCloudBounds>) -> PointCloudAssets {
    PointCloudAssets {
        position_texture: Handle::default(),
        colour_class_texture: Handle::default(),
        spatial_index_texture: Handle::default(),
        result_texture: Handle::default(),
        depth_texture: Handle::default(),
        heightmap_texture: Handle::default(),
        bounds,
        is_loaded: false,
    }
}

#[derive(Component)]
struct FpsText;

/// Setup renderer with unified texture loading
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
) {
    println!("=== GPU-ACCELERATED POINT CLOUD RENDERER (UNIFIED TEXTURES) ===");

    load_unified_textures(&asset_server, &mut assets);
    spawn_lighting(&mut commands);
    create_edl_post_processor_camera(&mut commands);
    spawn_ui(&mut commands);
    spawn_gizmos(&mut commands, &mut meshes, &mut materials, &asset_server);
}

/// Load unified texture set: position, colour+classification, heightmap
fn load_unified_textures(asset_server: &AssetServer, assets: &mut PointCloudAssets) {
    let position_texture_path = format!(
        "{}_position_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let colour_class_texture_path = format!(
        "{}_colour_class_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}_heightmap_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let spatial_index_texture_path = format!(
        "{}_spatial_index_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );

    println!("Loading unified DDS textures:");
    println!(
        "  Position: {} (RGBA32F XYZ + connectivity class id)",
        position_texture_path
    );
    println!(
        "  Colour+Class: {} (RGBA32F RGB + classification)",
        colour_class_texture_path
    );
    println!(
        "  Spatial Index: {} (RG32Uint cell_id + point_index)",
        spatial_index_texture_path
    );
    println!("  Heightmap: {} (R32F elevation)", heightmap_texture_path);

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.colour_class_texture = asset_server.load(&colour_class_texture_path);
    assets.depth_texture = Handle::default();
    assets.spatial_index_texture = asset_server.load(&spatial_index_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);
}

fn spawn_gizmos(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    create_direction_gizmo(commands, meshes, materials, asset_server);
}

fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
    ));
}

fn create_edl_post_processor_camera(commands: &mut Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        EDLSettings {
            radius: 4.0,
            strength: 100.0,
            ambient_boost: 0.8,
            contrast: 1.2,
        },
    ));
    commands.insert_resource(ViewportCamera::default());
}

fn spawn_ui(commands: &mut Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("FPS: "),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1., 0., 0.)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    right: Val::Px(12.0),
                    ..default()
                },
                FpsText,
            ));
        });
}

fn fps_text_update_system(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.0 = format!("FPS: {value:.1}");
            }
        }
    }
}

#[derive(Resource, Default)]
pub struct SelectionBuffer {
    pub selected_ids: Vec<u32>,
}

fn update_selection_buffer(
    mut selection_buffer: ResMut<SelectionBuffer>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        println!("add selection id");
        selection_buffer.selected_ids.push(2);
    }

    // Number keys 1-9 to set specific IDs
    if keyboard.just_pressed(KeyCode::Digit1) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(10);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(11);
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(12);
    }
    if keyboard.just_pressed(KeyCode::Digit4) {
        selection_buffer.selected_ids.clear();
        selection_buffer.selected_ids.push(13);
    }

    // Clear all selections
    if keyboard.just_pressed(KeyCode::KeyC) {
        selection_buffer.selected_ids.clear();
    }
}
