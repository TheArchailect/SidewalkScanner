use bevy::prelude::*;
#[derive(Resource, Default)]
pub struct LoadingProgress {
    pub bounds_loaded: bool,
    pub textures_loaded: bool,
    pub textures_loading_states: Vec<(String, i32)>,
    pub textures_configured: bool,
    pub point_cloud_created: bool,
    pub compute_pipelines_ready: bool,
}
