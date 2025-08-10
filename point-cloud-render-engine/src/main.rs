use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;

mod engine;
mod tools;

use engine::{
    camera::{ViewportCamera, camera_controller},
    gizmos::{create_direction_gizmo, update_direction_gizmo, update_mouse_intersection_gizmo},
    grid::GridCreated,
    point_cloud::check_textures_loaded,
    point_cloud::{PointCloudAssets, PointCloudBounds},
};

use tools::polygon::{
    PolygonCounter, PolygonTool, polygon_tool_system, update_polygon_preview, update_polygon_render,
};

use crate::engine::shaders::PointCloudShader;

const RELATIVE_ASSET_PATH: &'static str = "encoded_textures/warsaw";
const TEXTURE_RESOLUTION: &'static str = "2k";

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

fn create_app() -> App {
    let mut app = App::new();

    app.add_plugins(create_default_plugins())
        .add_plugins(MaterialPlugin::<PointCloudShader>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(JsonAssetPlugin::<PointCloudBounds>::new(&["bounds.json"]))
        .init_resource::<BoundsLoader>()
        .insert_resource(create_point_cloud_assets(None))
        .add_systems(Startup, setup)
        .add_systems(Update, (load_bounds_system, check_textures_loaded))
        .add_systems(Update, (fps_text_update_system, camera_controller))
        .add_systems(
            Update,
            (update_direction_gizmo, update_mouse_intersection_gizmo),
        )
        .add_systems(
            Update,
            (
                polygon_tool_system,
                update_polygon_preview,
                update_polygon_render,
            ),
        )
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<GridCreated>();

    app
}

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

fn get_bounds_path() -> String {
    format!("{}_bounds_{}.json", RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION)
}

fn create_default_plugins() -> impl PluginGroup {
    let window_config = WindowPlugin {
        primary_window: Some(create_window_config()),
        ..default()
    };

    let asset_config = AssetPlugin {
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

fn create_point_cloud_assets(bounds: Option<PointCloudBounds>) -> PointCloudAssets {
    PointCloudAssets {
        position_texture: Handle::default(),
        metadata_texture: Handle::default(),
        heightmap_texture: Handle::default(),
        bounds,
        is_loaded: false,
    }
}

#[derive(Component)]
struct FpsText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
) {
    println!("=== GPU-ACCELERATED POINT CLOUD RENDERER (DDS) ===");

    load_textures(&asset_server, &mut assets);
    // spawn_demo_objects(&mut commands, &mut meshes, &mut materials);
    spawn_lighting(&mut commands);
    spawn_camera_fallback(&mut commands);
    spawn_ui(&mut commands);
    spawn_gimos(commands, meshes, materials, asset_server);
}

fn load_textures(asset_server: &AssetServer, assets: &mut PointCloudAssets) {
    let position_texture_path = format!(
        "{}_positions_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let metadata_texture_path = format!(
        "{}_metadata_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}_road_heightmap_{}.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );

    println!("Loading DDS textures for fast GPU processing:");
    println!(
        "  Position: {} (16-bit float, BC6H-ready)",
        position_texture_path
    );
    println!("  Metadata: {} (32-bit float)", metadata_texture_path);

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.metadata_texture = asset_server.load(&metadata_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);
}

fn spawn_gimos(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    create_direction_gizmo(&mut commands, &mut meshes, &mut materials, &asset_server);
}

fn spawn_demo_objects(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // Circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
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

fn spawn_camera_fallback(commands: &mut Commands) {
    // Spawn default camera - will be updated once bounds are loaded
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
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
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(12.0),
                    left: Val::Px(12.0),
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
