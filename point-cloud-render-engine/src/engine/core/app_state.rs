// use crate::engine::camera::viewport_camera::{ViewportCamera, camera_controller};
use crate::engine::loading::progress::LoadingProgress;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use crate::rpc::web_rpc::WebRpcInterface;

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

// sent notification with the current progress to the front end
pub fn update_loading_frontend(
    loading_progress: Res<LoadingProgress>,
    mut rpc_interface: ResMut<WebRpcInterface>,
    mut last_send_time: Local<f32>,
    time: Res<Time>,
){
    // not to overwhelm the frontend
    let current_time = time.elapsed_secs();
    if current_time - *last_send_time <= 0.5 {return;}

    let mut progress = loading_progress.textures_loading_states.clone();
    
    // get the rest of the stuff in here too:
    progress.insert(0, (String::from("loading"),i32::from(true)));
    progress.insert(1, (String::from("Bounds"), i32::from(loading_progress.bounds_loaded)));
    progress.push((String::from("Configuring Textures"), i32::from(loading_progress.textures_configured)));
    progress.push((String::from("Creating Point Clouds"), i32::from(loading_progress.point_cloud_created)));
    progress.push((String::from("Computing pipelines"), i32::from(loading_progress.compute_pipelines_ready)));
    
    let loading_progress_json = serde_json::to_value(
        progress.into_iter().collect::<std::collections::HashMap<_, _>>()
    ).unwrap();

    rpc_interface.send_notification(
        "loading",
        loading_progress_json,
    );
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
pub fn transition_to_running(
    mut next_state: ResMut<NextState<AppState>>, 
    mut rpc_interface: ResMut<WebRpcInterface>,
) {
    println!("→ All systems ready, transitioning to Running state");
    rpc_interface.send_notification(
        "loading",
        serde_json::json!({"loading":0}),
    );
    next_state.set(AppState::Running);
}
