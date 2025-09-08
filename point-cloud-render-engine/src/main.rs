use bevy::prelude::*;

mod constants;
mod engine;
mod rpc;
mod tools;

use crate::constants::path::RELATIVE_ASSET_PATH;

use crate::engine::{
    point_cloud::{PointCloudAssets, SceneManifest},
    render_mode::RenderModeState,
};

use crate::engine::core::app_setup::create_app;

const TEXTURE_RESOLUTION: &'static str = "2048x2048";

fn main() {
    let mut app = create_app();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            app.run();
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.run();
    }
}

// Abstracted texture loading function
fn load_unified_textures(asset_server: &AssetServer, assets: &mut PointCloudAssets) {
    let position_texture_path =
        format!("{}{}/position.dds", RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION);
    let colour_class_texture_path = format!(
        "{}{}/colourclass.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}{}/heightmap.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );
    let spatial_index_texture_path = format!(
        "{}{}/spatialindex.dds",
        RELATIVE_ASSET_PATH, TEXTURE_RESOLUTION
    );

    println!("Loading unified DDS textures...");

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.colour_class_texture = asset_server.load(&colour_class_texture_path);
    assets.spatial_index_texture = asset_server.load(&spatial_index_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);
}
