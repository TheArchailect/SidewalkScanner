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

use crate::engine::loading::progress::LoadingProgress;
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States, Resource)]
pub enum AppState {
    #[default]
    Loading,
    AssetsLoaded,
    ComputePipelinesReady,
    Running,
}

#[derive(Component)]
pub struct FpsText;

#[derive(Resource, Default, Clone, ExtractResource)]
pub struct PipelineDebugState {
    pub entities_queued: u32,
    pub mesh_instances_found: u32,
    pub pipeline_specializations: u32,
    pub phase_items_added: u32,
    pub views_with_phases: u32,
}

// Transition to AssetsLoaded state
pub fn transition_to_assets_loaded(
    loading_progress: Res<LoadingProgress>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if loading_progress.point_cloud_created {
        println!("→ Transitioning to AssetsLoaded state");
        next_state.set(AppState::AssetsLoaded);
    }
}

pub fn transition_to_compute_ready(
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
pub fn transition_to_running(mut next_state: ResMut<NextState<AppState>>) {
    println!("→ All systems ready, transitioning to Running state");
    next_state.set(AppState::Running);
}
