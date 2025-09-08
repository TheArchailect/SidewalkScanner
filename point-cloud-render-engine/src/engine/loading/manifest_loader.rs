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
use crate::constants::path::RELATIVE_MANIFEST_PATH;
use crate::engine::assets::point_cloud_assets::create_point_cloud_assets;
use crate::engine::core::window_config::create_window_config;
use crate::rpc::web_rpc::WebRpcPlugin;

use crate::engine::loading::progress::LoadingProgress;
use crate::load_unified_textures;

#[derive(Resource, Default)]
pub struct ManifestLoader {
    handle: Option<Handle<SceneManifest>>,
}

// Start the loading process
pub fn start_loading(mut manifest_loader: ResMut<ManifestLoader>, asset_server: Res<AssetServer>) {
    // let bounds_path = get_bounds_path();
    let manifest_path = format!("{}/manifest.json", RELATIVE_MANIFEST_PATH);
    manifest_loader.handle = Some(asset_server.load(&manifest_path));
}

// Load bounds and start texture loading when ready
pub fn load_bounds_system(
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
