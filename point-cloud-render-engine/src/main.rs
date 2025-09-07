// Standard library and external crates
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;

// Local modules
mod constants;
mod engine;
mod rpc;
mod tools;

// Crate engine modules
use crate::engine::{
    camera::{ViewportCamera, camera_controller},
    compute_classification::{
        ComputeClassificationPlugin, ComputeClassificationState, run_classification_compute,
    },
    edl_compute_depth::{EDLComputePlugin, EDLRenderState, run_edl_compute},
    edl_post_processing::{EDLPostProcessPlugin, EDLSettings},
    gizmos::{create_direction_gizmo, update_direction_gizmo, update_mouse_intersection_gizmo},
    grid::{GridCreated, create_ground_grid},
    point_cloud::{
        PointCloud, PointCloudAssets, PointCloudBounds, SceneManifest, create_point_index_mesh,
    },
    point_cloud_render_pipeline::{
        PointCloudRenderPlugin, PointCloudRenderable, extract_point_cloud_render_state,
    },
    render_mode::{RenderModeState, render_mode_system},
};

// Crate tools modules
use crate::tools::{
    class_selection::{ClassSelectionState, handle_class_selection},
    polygon::{
        PolygonClassificationData, PolygonCounter, PolygonTool, polygon_tool_system,
        update_polygon_classification_shader, update_polygon_preview, update_polygon_render,
    },
    tool_manager::{
        PolygonActionEvent, ToolManager, ToolSelectionEvent, handle_polygon_action_events,
        handle_tool_keyboard_shortcuts, handle_tool_selection_events,
    },
};
// Create Web RPC modules
use rpc::web_rpc::WebRpcPlugin;

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
struct ManifestLoader {
    handle: Option<Handle<SceneManifest>>,
}

#[derive(Resource, Default)]
pub struct SelectionBuffer {
    pub selected_ids: Vec<u32>,
}

#[derive(Component)]
struct FpsText;
#[derive(Resource, Default, Clone, ExtractResource)]
pub struct PipelineDebugState {
    pub entities_queued: u32,
    pub mesh_instances_found: u32,
    pub pipeline_specializations: u32,
    pub phase_items_added: u32,
    pub views_with_phases: u32,
}

const RELATIVE_ASSET_PATH: &'static str = "output/riga_numbered_0.03/terrain/";
const RELATIVE_MANIFEST_PATH: &'static str = "output/riga_numbered_0.03";
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
        .add_plugins(PointCloudRenderPlugin)
        .init_resource::<PipelineDebugState>()
        .add_plugins(ExtractResourcePlugin::<PipelineDebugState>::default())
        .add_plugins(bevy::render::extract_component::ExtractComponentPlugin::<
            PointCloudRenderable,
        >::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        // Registers SceneManifest as a loadable asset type from JSON files.
        .add_plugins(JsonAssetPlugin::<SceneManifest>::new(&["json"]))
        // Automatically extracts SceneManifest resource from main world to render world.
        .add_plugins(ExtractResourcePlugin::<SceneManifest>::default())
        .add_plugins(ComputeClassificationPlugin)
        .add_plugins(EDLComputePlugin)
        .add_plugins(EDLPostProcessPlugin)
        .add_plugins(WebRpcPlugin);

    // Initialise resources early
    app.init_resource::<LoadingProgress>()
        .init_resource::<ManifestLoader>()
        .init_resource::<ClassSelectionState>()
        .init_resource::<SelectionBuffer>()
        .init_resource::<PolygonClassificationData>()
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<RenderModeState>()
        .init_resource::<GridCreated>()
        .init_resource::<ToolManager>()
        .add_event::<ToolSelectionEvent>()
        .add_event::<PolygonActionEvent>()
        .insert_resource(create_point_cloud_assets(None));

    // Configure render app with proper resource extraction
    if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
        // Initialise the resource in the render world
        render_app
            .init_resource::<State<AppState>>()
            .init_resource::<PipelineDebugState>();

        // Extract main-world state each frame
        render_app.add_systems(
            bevy::render::ExtractSchedule,
            (
                extract_app_state,
                extract_point_cloud_render_state,
                extract_camera_phases,
                extract_scene_manifest,
            ),
        );

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
        );

    // Base runtime systems that run on all platforms.
    let runtime_systems = (
        // Runtime systems - only run when everything is ready
        handle_class_selection,
        fps_notification_system,
        camera_controller,
        update_direction_gizmo,
        update_mouse_intersection_gizmo,
        // Tool management systems
        handle_tool_keyboard_shortcuts, // Native shortcuts or no-op for WASM
        handle_tool_selection_events,   // Process tool activation events
        handle_polygon_action_events,   // Process polygon action events
        // Tool-specific systems - run after tool state changes
        polygon_tool_system,
        update_polygon_preview,
        update_polygon_render,
        // Other systems
        render_mode_system,
        update_selection_buffer,
        update_polygon_classification_shader,
    );

    // Add fps_text_update_system only for native builds.
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_systems(Update, fps_text_update_system);
    }

    app.add_systems(Update, runtime_systems.run_if(in_state(AppState::Running)));

    app.add_systems(
        Update,
        debug_pipeline_state.run_if(in_state(AppState::Running)),
    );

    app
}

fn extract_camera_phases(
    mut point_cloud_phases: ResMut<
        bevy::render::render_phase::ViewSortedRenderPhases<
            engine::point_cloud_render_pipeline::PointCloudPhase,
        >,
    >,
    cameras: bevy::render::Extract<Query<(Entity, &Camera), With<Camera3d>>>,
    mut live_entities: Local<std::collections::HashSet<bevy::render::view::RetainedViewEntity>>,
) {
    live_entities.clear();
    for (main_entity, camera) in &cameras {
        if !camera.is_active {
            continue;
        }

        let retained_view_entity =
            bevy::render::view::RetainedViewEntity::new(main_entity.into(), None, 0);
        point_cloud_phases.insert_or_clear(retained_view_entity);
        live_entities.insert(retained_view_entity);
    }
    point_cloud_phases.retain(|camera_entity, _| live_entities.contains(camera_entity));
}

fn fps_notification_system(
    mut rpc_interface: ResMut<rpc::web_rpc::WebRpcInterface>,
    diagnostics: Res<DiagnosticsStore>,
    mut last_send_time: Local<f32>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Send FPS every 0.5 seconds
    if current_time - *last_send_time >= 0.5 {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                rpc_interface.send_notification(
                    "fps_update",
                    serde_json::json!({
                        "fps": value as f32
                    }),
                );
                *last_send_time = current_time;
            }
        }
    }
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

    #[cfg(not(target_arch = "wasm32"))]
    {
        create_native_overlays(&mut commands);
    }
}

fn extract_scene_manifest(
    mut commands: Commands,
    assets: bevy::render::Extract<Res<PointCloudAssets>>,
    manifests: bevy::render::Extract<Res<Assets<SceneManifest>>>,
) {
    // Extract manifest once when available.
    if let Some(ref handle) = assets.manifest {
        if let Some(manifest) = manifests.get(handle) {
            commands.insert_resource(manifest.clone());
        }
    }
}

// Start the loading process
fn start_loading(mut manifest_loader: ResMut<ManifestLoader>, asset_server: Res<AssetServer>) {
    // let bounds_path = get_bounds_path();
    let manifest_path = format!("{}/manifest.json", RELATIVE_MANIFEST_PATH);
    manifest_loader.handle = Some(asset_server.load(&manifest_path));
}

// Load bounds and start texture loading when ready
fn load_bounds_system(
    mut loading_progress: ResMut<LoadingProgress>,
    manifest_loader: ResMut<ManifestLoader>,
    mut assets: ResMut<PointCloudAssets>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<SceneManifest>>,
) {
    if loading_progress.bounds_loaded {
        return;
    }

    if let Some(ref handle) = manifest_loader.handle {
        if let Some(manifest) = manifests.get(handle) {
            println!("✓ Bounds loaded successfully");
            assets.manifest = Some(handle.clone());
            commands.insert_resource(manifest.clone());
            loading_progress.bounds_loaded = true;

            // Update camera with bounds
            let bounds = manifest.to_point_cloud_bounds();
            let vp_camera = ViewportCamera::with_bounds(&bounds);
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

        // Ensure result texture uses non-filtering sampler.
        result_image.sampler =
            bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            });

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

        // Add sampler configuration to depth texture.
        depth_image.sampler =
            bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            });

        assets.result_texture = images.add(result_image);
        assets.depth_texture = images.add(depth_image);

        println!("✓ Compute-ready textures created with proper formats");
        loading_progress.textures_configured = true;
    }
}

fn create_point_cloud_when_ready(
    mut loading_progress: ResMut<LoadingProgress>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    mut assets: ResMut<PointCloudAssets>,
    images: ResMut<Assets<Image>>,
    mut grid_created: ResMut<GridCreated>,
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<SceneManifest>>,
) {
    if loading_progress.point_cloud_created || !loading_progress.textures_configured {
        return;
    }

    let Some(bounds) = &assets.get_bounds(&manifests) else {
        return;
    };

    // Create point cloud entity without material - custom pipeline handles rendering.
    spawn_point_cloud_entity(&mut commands, &mut meshes, bounds);

    // Create grid with standard material pipeline.
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
        println!("Grid created");
    }

    // Create gizmos using standard material system.
    spawn_gizmos(
        &mut commands,
        &mut meshes,
        &mut standard_materials,
        &asset_server,
    );

    assets.is_loaded = true;
    loading_progress.point_cloud_created = true;
    println!("Point cloud and visual elements ready");
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
    let position_texture_path =
        format!("{}{}/position.dds", RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION);
    let colour_class_texture_path = format!(
        "{}{}/colourclass.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}{}/heightmap.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let spatial_index_texture_path = format!(
        "{}{}/spatialindex.dds",
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
        &assets.result_texture,
        &assets.depth_texture,
    ] {
        if let Some(image) = images.get_mut(texture_handle) {
            image.sampler = sampler_config.clone();
        }
    }
}

fn spawn_point_cloud_entity(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    bounds: &PointCloudBounds,
) {
    // Create indexed vertex buffer for GPU-side point expansion.
    let mesh = create_point_index_mesh(bounds.loaded_points);

    commands.spawn((
        // Standard 3D mesh component without material binding.
        Mesh3d(meshes.add(mesh)),
        Transform::from_translation(Vec3::ZERO),
        Visibility::Visible,
        InheritedVisibility::VISIBLE,
        ViewVisibility::default(),
        GlobalTransform::default(),
        // Point cloud identification for systems and queries.
        PointCloud,
        // Custom render pipeline component containing vertex count for draw calls.
        PointCloudRenderable {
            point_count: bounds.loaded_points as u32,
        },
        // Disable frustum culling for large-scale point cloud rendering.
        bevy::render::view::NoFrustumCulling,
    ));

    println!(
        "Point cloud entity spawned with {} vertices using custom render pipeline",
        bounds.loaded_points
    );
}

// fn get_bounds_path() -> String {
//     format!(
//         "{}_metadata_{}.json",
//         RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
//     )
// }

pub fn debug_pipeline_state(
    debug_state: Res<PipelineDebugState>,
    point_cloud_query: Query<Entity, With<PointCloudRenderable>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::F1) {
        let pc_entities = point_cloud_query.iter().count();

        println!("=== PIPELINE DEBUG STATE ===");
        println!("Point cloud entities in main world: {}", pc_entities);
        println!(
            "Entities queued for rendering: {}",
            debug_state.entities_queued
        );
        println!("Mesh instances found: {}", debug_state.mesh_instances_found);
        println!(
            "Pipeline specializations: {}",
            debug_state.pipeline_specializations
        );
        println!("Phase items added: {}", debug_state.phase_items_added);
        println!("Views with phases: {}", debug_state.views_with_phases);

        // Critical assertions
        assert!(pc_entities > 0, "No point cloud entities in main world");
        assert!(
            debug_state.entities_queued > 0,
            "No entities queued for rendering"
        );
        assert!(
            debug_state.mesh_instances_found > 0,
            "No mesh instances found"
        );
        assert!(debug_state.phase_items_added > 0, "No phase items added");
        assert!(debug_state.views_with_phases > 0, "No views have phases");

        println!("All assertions passed!");
    }
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

fn create_point_cloud_assets(manifest: Option<Handle<SceneManifest>>) -> PointCloudAssets {
    PointCloudAssets {
        position_texture: Handle::default(),
        colour_class_texture: Handle::default(),
        spatial_index_texture: Handle::default(),
        result_texture: Handle::default(),
        depth_texture: Handle::default(),
        heightmap_texture: Handle::default(),
        // Asset textures are None until manifest confirms asset atlas presence.
        asset_position_texture: None,
        asset_colour_class_texture: None,
        manifest,
        is_loaded: false,
    }
}
// UI and lighting systems remain unchanged
fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
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

fn create_native_overlays(commands: &mut Commands) {
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
