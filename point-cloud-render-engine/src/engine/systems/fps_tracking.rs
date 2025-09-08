// Standard library and external crates
use bevy::asset::AssetMetaCheck;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::window::PresentMode;
use bevy_common_assets::json::JsonAssetPlugin;

// Crate engine modules
use crate::engine::loading::point_cloud_creator::create_point_cloud_when_ready;
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

use crate::engine::loading::texture_config::configure_loaded_textures;
// Crate tools modules
use crate::engine::core::app_state::{AppState, PipelineDebugState};
use crate::engine::loading::manifest_loader::{ManifestLoader, load_bounds_system, start_loading};
use crate::engine::loading::texture_loader::check_texture_loading;
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
use crate::engine::loading::progress::LoadingProgress;
use crate::rpc::web_rpc::WebRpcPlugin;

// Transitions
use crate::engine::core::app_state::{
    transition_to_assets_loaded, transition_to_compute_ready, transition_to_running,
};

// Extraction
use crate::engine::render::extraction::{
    app_state::extract_app_state, camera_phases::extract_camera_phases,
    render_state::extract_point_cloud_render_state, scene_manifest::extract_scene_manifest,
};

use crate::engine::core::app_state::FpsText;
use crate::rpc::web_rpc::WebRpcInterface;

pub fn fps_notification_system(
    mut rpc_interface: ResMut<WebRpcInterface>,
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

pub fn fps_text_update_system(
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
