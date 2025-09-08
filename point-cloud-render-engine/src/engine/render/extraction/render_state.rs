use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::point_cloud_render_pipeline::PointCloudRenderState;
use bevy::prelude::*;

/// System to extract point cloud render state from main world to render world.
/// Captures camera position and bounds data for GPU uniform buffer creation.
pub fn extract_point_cloud_render_state(
    mut commands: Commands,
    camera_query: bevy::render::Extract<Query<&GlobalTransform, With<Camera3d>>>,
    assets: bevy::render::Extract<Res<PointCloudAssets>>,
    manifests: bevy::render::Extract<Res<Assets<SceneManifest>>>,
) {
    let mut render_state = PointCloudRenderState::default();

    if let Ok(camera_transform) = camera_query.single() {
        render_state.camera_position = camera_transform.translation();
        render_state.should_render = true;
    }

    let Some(bounds) = assets.get_bounds(&manifests) else {
        return;
    };

    render_state.bounds_min = Vec3::new(
        bounds.min_x() as f32,
        bounds.min_y() as f32,
        bounds.min_z() as f32,
    );
    render_state.bounds_max = Vec3::new(
        bounds.max_x() as f32,
        bounds.max_y() as f32,
        bounds.max_z() as f32,
    );
    render_state.texture_size = bounds.texture_size as f32;

    commands.insert_resource(render_state);
}
