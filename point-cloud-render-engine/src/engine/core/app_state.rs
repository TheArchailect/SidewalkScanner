// use crate::engine::camera::viewport_camera::{ViewportCamera, camera_controller};
use crate::engine::loading::progress::LoadingProgress;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

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
