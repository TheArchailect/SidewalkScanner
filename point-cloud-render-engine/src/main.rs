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
use crate::engine::grid::create_ground_grid;
use crate::engine::point_cloud::PointCloud;
use crate::engine::point_cloud::create_point_index_mesh;
use crate::engine::point_cloud::update_camera_uniform;
use crate::engine::shaders::PointCloudShader;
use engine::compute_classification::ComputeClassificationPlugin;
use engine::render_mode::{RenderModeState, render_mode_system};
use engine::{
    camera::{ViewportCamera, camera_controller},
    gizmos::{create_direction_gizmo, update_direction_gizmo, update_mouse_intersection_gizmo},
    grid::GridCreated,
    point_cloud::{PointCloudAssets, PointCloudBounds},
};
use tools::polygon::{
    PolygonClassificationData, PolygonCounter, PolygonTool, polygon_tool_system,
    update_polygon_classification_shader, update_polygon_preview, update_polygon_render,
};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Resource)]
pub enum AppState {
    #[default]
    Loading,
    AssetsLoaded,
    ComputePipelinesReady,
    Running,
}
#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub bounds_loaded: bool,
    pub textures_loaded: bool,
    pub textures_configured: bool,
    pub point_cloud_created: bool,
    pub compute_pipelines_ready: bool,
}
#[derive(Resource, Default)]
struct BoundsLoader {
    handle: Option<Handle<PointCloudBounds>>,
}
#[derive(Resource, Default)]
pub struct SelectionBuffer {
    pub selected_ids: Vec<u32>,
}
#[derive(Component)]
struct FpsText;

const RELATIVE_ASSET_PATH: &'static str = "pre_processor_data/riga_versions/riga_numbered_0.03";
const TEXTURE_RESOLUTION: &'static str = "2048x2048";

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
        .init_state::<AppState>()
        .add_plugins(MaterialPlugin::<PointCloudShader>::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(JsonAssetPlugin::<PointCloudBounds>::new(&["json"]))
        .add_plugins(ComputeClassificationPlugin)
        .add_plugins(EDLComputePlugin)
        .add_plugins(EDLPostProcessPlugin);

    // Initialise resources early
    app.init_resource::<LoadingProgress>()
        .init_resource::<BoundsLoader>()
        .init_resource::<ClassSelectionState>()
        .init_resource::<SelectionBuffer>()
        .init_resource::<PolygonClassificationData>()
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<RenderModeState>()
        .init_resource::<GridCreated>()
        .insert_resource(create_point_cloud_assets(None));

    // Configure render app with proper resource extraction
    if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
        // Initialise the resource in the render world
        render_app.init_resource::<State<AppState>>();

        // Extract main-world state each frame
        render_app.add_systems(bevy::render::ExtractSchedule, extract_app_state);

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
                    .in_set(bevy::render::RenderSet::Queue)
                    .run_if(in_state(AppState::Running)),
            );
    }

    // State-based system scheduling
    app.add_systems(Startup, (setup, start_loading).chain())
        .add_systems(
            Update,
            (
                // Loading phase systems
                load_bounds_system,
                check_texture_loading,
                configure_loaded_textures,
                create_point_cloud_when_ready,
                transition_to_assets_loaded,
            )
                .chain()
                .run_if(in_state(AppState::Loading)),
        )
        .add_systems(
            Update,
            transition_to_compute_ready.run_if(in_state(AppState::AssetsLoaded)),
        )
        .add_systems(
            Update,
            transition_to_running.run_if(in_state(AppState::ComputePipelinesReady)),
        )
        .add_systems(
            Update,
            (
                // Runtime systems - only run when everything is ready
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
            )
                .run_if(in_state(AppState::Running)),
        );

    app
}

fn extract_app_state(
    main_world: bevy::render::Extract<Res<State<AppState>>>,
    mut commands: Commands,
) {
    commands.insert_resource(State::new(*main_world.get()));
}

// Startup system that only handles basic initialisation
fn setup(mut commands: Commands) {
    spawn_lighting(&mut commands);
    create_edl_post_processor_camera(&mut commands);
    spawn_ui(&mut commands);
}

// Start the loading process
fn start_loading(mut bounds_loader: ResMut<BoundsLoader>, asset_server: Res<AssetServer>) {
    let bounds_path = get_bounds_path();
    bounds_loader.handle = Some(asset_server.load(&bounds_path));
}

// Load bounds and start texture loading when ready
fn load_bounds_system(
    mut loading_progress: ResMut<LoadingProgress>,
    bounds_loader: ResMut<BoundsLoader>,
    mut assets: ResMut<PointCloudAssets>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bounds_assets: Res<Assets<PointCloudBounds>>,
) {
    if loading_progress.bounds_loaded {
        return;
    }

    if let Some(ref handle) = bounds_loader.handle {
        if let Some(bounds) = bounds_assets.get(handle) {
            println!("✓ Bounds loaded successfully");
            assets.bounds = Some(bounds.clone());
            loading_progress.bounds_loaded = true;

            // Update camera with bounds
            let vp_camera = ViewportCamera::with_bounds(bounds);
            commands.insert_resource(vp_camera);

            // Start loading textures now that we have bounds
            load_unified_textures(&asset_server, &mut assets);
        }
    }
}

// Check if all required textures are loaded
fn check_texture_loading(
    mut loading_progress: ResMut<LoadingProgress>,
    assets: Res<PointCloudAssets>,
    asset_server: Res<AssetServer>,
) {
    if loading_progress.textures_loaded || !loading_progress.bounds_loaded {
        return;
    }

    let pos_loaded = matches!(
        asset_server.get_load_state(&assets.position_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let colour_class_loaded = matches!(
        asset_server.get_load_state(&assets.colour_class_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let heightmap_loaded = matches!(
        asset_server.get_load_state(&assets.heightmap_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let spatial_loaded = matches!(
        asset_server.get_load_state(&assets.spatial_index_texture),
        Some(bevy::asset::LoadState::Loaded)
    );

    if pos_loaded && colour_class_loaded && spatial_loaded && heightmap_loaded {
        println!("✓ All DDS textures loaded successfully");
        loading_progress.textures_loaded = true;
    }
}

// Configure texture sampling and create compute-ready textures
fn configure_loaded_textures(
    mut loading_progress: ResMut<LoadingProgress>,
    mut assets: ResMut<PointCloudAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    if loading_progress.textures_configured || !loading_progress.textures_loaded {
        return;
    }
    configure_texture_sampling(&mut images, &assets);

    // Create compute-ready textures with proper formats
    if let Some(original_image) = images.get(&assets.colour_class_texture).cloned() {
        // Create result texture with proper storage binding usage
        let mut result_image = original_image;
        result_image.texture_descriptor.format =
            bevy::render::render_resource::TextureFormat::Rgba32Float;
        result_image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;

        // Create depth texture
        let mut depth_image = Image::new_uninit(
            bevy::render::render_resource::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            bevy::render::render_resource::TextureFormat::R32Float,
            bevy::asset::RenderAssetUsages::RENDER_WORLD,
        );
        depth_image.texture_descriptor.usage =
            bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::STORAGE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_SRC
                | bevy::render::render_resource::TextureUsages::COPY_DST;

        assets.result_texture = images.add(result_image);
        assets.depth_texture = images.add(depth_image);

        println!("✓ Compute-ready textures created with proper formats");
        loading_progress.textures_configured = true;
    }
}

// Create point cloud and grid when textures are ready
fn create_point_cloud_when_ready(
    mut loading_progress: ResMut<LoadingProgress>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PointCloudShader>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
    images: ResMut<Assets<Image>>,
    mut grid_created: ResMut<GridCreated>,
    asset_server: Res<AssetServer>,
) {
    if loading_progress.point_cloud_created || !loading_progress.textures_configured {
        return;
    }

    let Some(bounds) = &assets.bounds.clone() else {
        return;
    };

    // Create point cloud
    let mesh = create_point_index_mesh(bounds.loaded_points);
    let material = create_point_cloud_material(bounds, &assets);

    spawn_point_cloud_entity(
        &mut commands,
        &mut meshes,
        &mut materials,
        mesh,
        material,
        bounds,
    );

    // Create grid
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
        println!("✓ Grid created");
    }

    // Create gizmos
    spawn_gizmos(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &asset_server,
    );

    assets.is_loaded = true;
    loading_progress.point_cloud_created = true;
    println!("✓ Point cloud and visual elements ready");
}

// Transition to AssetsLoaded state
fn transition_to_assets_loaded(
    loading_progress: Res<LoadingProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if loading_progress.point_cloud_created {
        println!("→ Transitioning to AssetsLoaded state");
        next_state.set(AppState::AssetsLoaded);
    }
}

fn transition_to_compute_ready(
    mut loading_progress: ResMut<LoadingProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if loading_progress.point_cloud_created && !loading_progress.compute_pipelines_ready {
        loading_progress.compute_pipelines_ready = true;
        println!("→ Transitioning to ComputePipelinesReady state");
        next_state.set(AppState::ComputePipelinesReady);
    }
}

// Final transition to running state
fn transition_to_running(mut next_state: ResMut<NextState<AppState>>) {
    println!("→ All systems ready, transitioning to Running state");
    next_state.set(AppState::Running);
}

// Abstracted texture loading function
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

    println!("Loading unified DDS textures...");

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.colour_class_texture = asset_server.load(&colour_class_texture_path);
    assets.spatial_index_texture = asset_server.load(&spatial_index_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);
}

// Abstracted texture configuration function
fn configure_texture_sampling(images: &mut ResMut<Assets<Image>>, assets: &PointCloudAssets) {
    use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

    let sampler_config = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });

    for texture_handle in [
        &assets.position_texture,
        &assets.colour_class_texture,
        &assets.spatial_index_texture,
    ] {
        if let Some(image) = images.get_mut(texture_handle) {
            image.sampler = sampler_config.clone();
        }
    }
}

// Abstracted point cloud material creation
fn create_point_cloud_material(
    bounds: &PointCloudBounds,
    assets: &PointCloudAssets,
) -> PointCloudShader {
    PointCloudShader {
        position_texture: assets.position_texture.clone(),
        final_texture: assets.result_texture.clone(),
        depth_texture: assets.depth_texture.clone(),
        params: [
            Vec4::new(
                bounds.min_x() as f32,
                bounds.min_y() as f32,
                bounds.min_z() as f32,
                bounds.texture_size as f32,
            ),
            Vec4::new(
                bounds.max_x() as f32,
                bounds.max_y() as f32,
                bounds.max_z() as f32,
                0.0,
            ),
            Vec4::new(0.0, 0.0, 0.0, 0.0),
        ],
    }
}

// Abstracted point cloud entity spawning
fn spawn_point_cloud_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<PointCloudShader>>,
    mesh: Mesh,
    material: PointCloudShader,
    bounds: &PointCloudBounds,
) {
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(material)),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        GlobalTransform::default(),
        PointCloud,
        bevy::render::view::NoFrustumCulling,
    ));

    println!(
        "✓ Point cloud entity spawned with {} vertices",
        bounds.loaded_points
    );
}

// Rest of the helper functions remain the same...
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
        colour_class_texture: Handle::default(),
        spatial_index_texture: Handle::default(),
        result_texture: Handle::default(),
        depth_texture: Handle::default(),
        heightmap_texture: Handle::default(),
        bounds,
        is_loaded: false,
    }
}

// UI and lighting systems remain unchanged
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

fn spawn_gizmos(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    create_direction_gizmo(commands, meshes, materials, asset_server);
}

// FPS system remains unchanged
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

// Selection buffer system remains unchanged
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
