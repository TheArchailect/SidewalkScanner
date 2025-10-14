use crate::engine::core::app_state::FpsText;
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::view::RenderLayers;
use bevy_common_assets::json::JsonAssetPlugin;
// Crate engine modules
use crate::engine::camera::viewport_camera::camera_controller;
use crate::engine::loading::point_cloud_creator::create_point_cloud_when_ready;
use crate::engine::loading::texture_config::configure_loaded_textures;
use crate::engine::scene::gizmos::{update_direction_gizmo, update_mouse_intersection_gizmo};
use crate::engine::scene::grid::GridCreated;
use crate::engine::systems::debug_pipeline::debug_pipeline_state;
use crate::engine::systems::fps_tracking::fps_notification_system;
use crate::engine::{
    compute::compute_classification::{
        ComputeClassificationPlugin, ComputeClassificationState, run_classification_compute,
    },
    compute::edl_compute_depth::{EDLComputePlugin, EDLRenderState, run_edl_compute},
    render::edl_post_processing::EDLPostProcessPlugin,
    render::pipeline::point_cloud_render_pipeline::{PointCloudRenderPlugin, PointCloudRenderable},
    systems::render_mode::{MouseEnterObjectState, RenderModeState, render_mode_system},
};
// Crate tools modules
use crate::engine::core::app_state::{AppState, PipelineDebugState};
use crate::engine::loading::manifest_loader::{ManifestLoader, load_bounds_system, start_loading};
use crate::engine::loading::texture_loader::check_texture_loading;
use crate::tools::{
    asset_manager::AssetManagerPlugin,
    class_selection::{
        ClassSelectionState, SelectionBuffer, handle_class_selection, update_selection_buffer,
    },
    measure::{MeasureTool, measure_tool_system, update_measure_render},
    polygon::{
        PolygonClassificationData, PolygonCounter, PolygonHideRequestEvent, PolygonTool,
        PolygonToolPlugin, polygon_tool_system, update_polygon_classification_shader,
        update_polygon_preview, update_polygon_render,
    },
    tool_manager::{
        AssetPlacementEvent, ClearToolEvent, PolygonActionEvent, ToolManager, ToolSelectionEvent,
        handle_asset_placement_events, handle_clear_tool_events, handle_polygon_action_events,
        handle_tool_keyboard_shortcuts, handle_tool_selection_events,
    },
};
// Create Web RPC modules
use crate::engine::assets::point_cloud_assets::create_point_cloud_assets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::core::window_config::create_window_config;
use crate::engine::loading::progress::LoadingProgress;
use crate::rpc::web_rpc::WebRpcPlugin;
// Transitions
use crate::engine::core::app_state::{
    transition_to_assets_loaded, transition_to_compute_ready, transition_to_running,
    update_loading_frontend,
};

use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::systems::fps_tracking::fps_text_update_system;
// Extraction
use crate::engine::render::extraction::{
    app_state::extract_app_state, camera_phases::extract_camera_phases,
    render_state::extract_point_cloud_render_state, scene_manifest::extract_scene_manifest,
};
use crate::engine::render::instanced_render_plugin::InstancedAssetRenderPlugin;
use crate::engine::render::pipeline::point_cloud_render_pipeline::PointCloudRenderState;
use crate::tools::asset_manager::PlacedAssetInstances;

#[cfg(not(target_arch = "wasm32"))]
use crate::tools::tool_manager::clear_tool_on_escape;

pub fn create_app() -> App {
    let mut app = App::new();

    app.add_plugins(create_default_plugins())
        .init_state::<AppState>()
        .add_plugins(PointCloudRenderPlugin)
        .add_plugins(InstancedAssetRenderPlugin)
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
        .add_plugins(WebRpcPlugin)
        .add_plugins(WireframePlugin::default())
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::WHITE,
        });

    // Plugin for asset manager UI panel
    app.add_plugins(AssetManagerPlugin);

    // Plugin for Polygon
    app.add_plugins(PolygonToolPlugin);

    // Initialise resources early
    app.init_resource::<LoadingProgress>()
        .init_resource::<ManifestLoader>()
        .init_resource::<ClassSelectionState>()
        .init_resource::<SelectionBuffer>()
        .init_resource::<PolygonClassificationData>()
        .init_resource::<PolygonCounter>()
        .init_resource::<PolygonTool>()
        .init_resource::<MeasureTool>()
        .init_resource::<RenderModeState>()
        .init_resource::<MouseEnterObjectState>()
        .init_resource::<PlacedAssetInstances>()
        .init_resource::<GridCreated>()
        .init_resource::<ToolManager>()
        .add_event::<ToolSelectionEvent>()
        .add_event::<PolygonActionEvent>()
        .add_event::<AssetPlacementEvent>()
        .add_event::<ClearToolEvent>()
        .add_event::<PolygonHideRequestEvent>()
        .add_event::<PolygonHideRequestEvent>()
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
            .init_resource::<MouseEnterObjectState>()
            .init_resource::<PointCloudRenderState>()
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
                update_loading_frontend,
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
        handle_clear_tool_events,       // Apply clear tool events
        handle_polygon_action_events,   // Process polygon action events
        handle_asset_placement_events,  // Process asset placement events - NEW!
        // Tool-specific systems - run after tool state changes
        polygon_tool_system,
        update_polygon_preview,
        update_polygon_render,
        measure_tool_system,
        update_measure_render,
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

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_systems(
            Update,
            clear_tool_on_escape.run_if(in_state(AppState::Running)),
        );
    }

    app.add_systems(
        Update,
        debug_pipeline_state.run_if(in_state(AppState::Running)),
    );

    app
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
    use crate::constants::render_settings::EDL_SETTINGS;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        EDL_SETTINGS,
        RenderLayers::default().with(1),
    ));
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
