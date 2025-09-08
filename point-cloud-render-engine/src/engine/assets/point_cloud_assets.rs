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
use crate::rpc::web_rpc::WebRpcPlugin;

use crate::engine::core::window_config::create_window_config;

pub fn create_point_cloud_assets(manifest: Option<Handle<SceneManifest>>) -> PointCloudAssets {
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
