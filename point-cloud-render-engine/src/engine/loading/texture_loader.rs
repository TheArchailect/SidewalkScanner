use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::loading::progress::LoadingProgress;
use bevy::prelude::*;

// Check if all required textures are loaded
pub fn check_texture_loading(
    mut loading_progress: ResMut<LoadingProgress>,
    assets: Res<PointCloudAssets>,
    asset_server: Res<AssetServer>,
) {
    if loading_progress.textures_loaded || !loading_progress.bounds_loaded {
        return;
    }

    let pos_loaded = matches!(
        asset_server.get_load_state(&assets.position_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let colour_class_loaded = matches!(
        asset_server.get_load_state(&assets.colour_class_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let heightmap_loaded = matches!(
        asset_server.get_load_state(&assets.heightmap_texture),
        Some(bevy::asset::LoadState::Loaded)
    );
    let spatial_loaded = matches!(
        asset_server.get_load_state(&assets.spatial_index_texture),
        Some(bevy::asset::LoadState::Loaded)
    );

    //update the progress so we can send states to the frontend.
    //TODO: real percentages 

    let progress = vec![
        (String::from("Position texture"), i32::from(pos_loaded)),
        (String::from("Colour texture"), i32::from(colour_class_loaded)),
        (String::from("Heightmap"), i32::from(heightmap_loaded)),
        (String::from("Spatial index"), i32::from(spatial_loaded)),
    ];
    loading_progress.textures_loading_states = progress;
    
    if pos_loaded && colour_class_loaded && spatial_loaded && heightmap_loaded {
        println!("✓ All DDS textures loaded successfully");
        loading_progress.textures_loaded = true;
    }
}
