// Standard library and external crates
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;

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
    point_cloud_render_pipeline::{PointCloudRenderPlugin, PointCloudRenderable},
    render_mode::{RenderModeState, render_mode_system},
};

// Crate tools modules
use crate::engine::core::app_state::{AppState, PipelineDebugState};
use crate::tools::{
    class_selection::{ClassSelectionState, SelectionBuffer, handle_class_selection},
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
use crate::engine::assets::point_cloud_assets::create_point_cloud_assets;
use crate::engine::core::window_config::create_window_config;
use crate::rpc::web_rpc::WebRpcPlugin;

// Loading
use crate::engine::loading::manifest_loader::load_bounds_system;
use crate::engine::loading::manifest_loader::start_loading;

// Extraction
use crate::engine::loading::progress::LoadingProgress;
use crate::engine::render::extraction::{
    app_state::extract_app_state, camera_phases::extract_camera_phases,
    render_state::extract_point_cloud_render_state, scene_manifest::extract_scene_manifest,
};
// Check if all required textures are loaded
pub fn check_texture_loading(
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
