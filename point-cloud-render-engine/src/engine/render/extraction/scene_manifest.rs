use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use bevy::prelude::*;

pub fn extract_scene_manifest(
    mut commands: Commands,
    assets: bevy::render::Extract<Res<PointCloudAssets>>,
    manifests: bevy::render::Extract<Res<Assets<SceneManifest>>>,
) {
    // Extract manifest once when available.
    if let Some(ref handle) = assets.manifest {
        if let Some(manifest) = manifests.get(handle) {
            commands.insert_resource(manifest.clone());
        }
    }
}
