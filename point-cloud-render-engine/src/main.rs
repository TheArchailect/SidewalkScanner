use bevy::prelude::*;

mod constants;
mod engine;
mod rpc;
mod tools;

use crate::constants::path::{ASSET_PATH, RELATIVE_MANIFEST_PATH, TERRAIN_PATH};
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::core::app_setup::create_app;
use crate::engine::systems::render_mode::RenderModeState;

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
    let position_texture_path = format!(
        "{}{}{}/position.dds",
        RELATIVE_MANIFEST_PATH, TERRAIN_PATH, TEXTURE_RESOLUTION
    );
    let colour_class_texture_path = format!(
        "{}{}{}/colourclass.dds",
        RELATIVE_MANIFEST_PATH, TERRAIN_PATH, TEXTURE_RESOLUTION
    );
    let heightmap_texture_path = format!(
        "{}{}{}/heightmap.dds",
        RELATIVE_MANIFEST_PATH, TERRAIN_PATH, TEXTURE_RESOLUTION
    );
    let spatial_index_texture_path = format!(
        "{}{}{}/spatialindex.dds",
        RELATIVE_MANIFEST_PATH, TERRAIN_PATH, TEXTURE_RESOLUTION
    );

    let atlas_position_texture_path = format!(
        "{}{}{}/position.dds",
        RELATIVE_MANIFEST_PATH, ASSET_PATH, TEXTURE_RESOLUTION
    );
    let atlas_colourclass_texture_path = format!(
        "{}{}{}/colourclass.dds",
        RELATIVE_MANIFEST_PATH, ASSET_PATH, TEXTURE_RESOLUTION
    );

    println!("Loading unified DDS textures...");

    assets.position_texture = asset_server.load(&position_texture_path);
    assets.colour_class_texture = asset_server.load(&colour_class_texture_path);
    assets.spatial_index_texture = asset_server.load(&spatial_index_texture_path);
    assets.heightmap_texture = asset_server.load(&heightmap_texture_path);
    // Atlas's
    assets.asset_position_texture = asset_server.load(&atlas_position_texture_path);
    assets.asset_colour_class_texture = asset_server.load(&atlas_colourclass_texture_path);
}
