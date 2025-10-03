use crate::constants::path::RELATIVE_MANIFEST_PATH;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::camera::viewport_camera::ViewportCamera;
use crate::engine::loading::progress::LoadingProgress;
use crate::load_unified_textures;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ManifestLoader {
    handle: Option<Handle<SceneManifest>>,
}

// Start the loading process
pub fn start_loading(mut manifest_loader: ResMut<ManifestLoader>, asset_server: Res<AssetServer>) {
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
