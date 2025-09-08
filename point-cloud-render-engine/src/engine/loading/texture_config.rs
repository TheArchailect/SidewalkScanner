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
use crate::engine::core::app_state::{AppState, PipelineDebugState};
use crate::engine::loading::progress::LoadingProgress;
use crate::tools::{
    class_selection::{ClassSelectionState, SelectionBuffer, handle_class_selection},
    polygon::{
        PolygonClassificationData, PolygonCounter, PolygonTool, polygon_tool_system,
        update_polygon_classification_shader, update_polygon_preview, update_polygon_render,
    },
    tool_manager::{
        PolygonActionEvent, ToolManager, ToolSelectionEvent, handle_polygon_action_events,
        handle_tool_keyboard_shortcuts, handle_tool_selection_events,
    },
};
// Create Web RPC modules
use crate::engine::assets::point_cloud_assets::create_point_cloud_assets;
use crate::engine::core::window_config::create_window_config;
use crate::rpc::web_rpc::WebRpcPlugin;

// Loading
use crate::engine::loading::manifest_loader::load_bounds_system;
use crate::engine::loading::manifest_loader::start_loading;

// Extraction
use crate::engine::render::extraction::{
    app_state::extract_app_state, camera_phases::extract_camera_phases,
    render_state::extract_point_cloud_render_state, scene_manifest::extract_scene_manifest,
};

// Configure texture sampling and create compute-ready textures
pub fn configure_loaded_textures(
    mut loading_progress: ResMut<LoadingProgress>,
    mut assets: ResMut<PointCloudAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    if loading_progress.textures_configured || !loading_progress.textures_loaded {
        return;
    }
    configure_texture_sampling(&mut images, &assets);

    // Create compute-ready textures with proper formats
    if let Some(original_image) = images.get(&assets.colour_class_texture).cloned() {
        // Create result texture with proper storage binding usage
        let mut result_image = original_image;
        result_image.texture_descriptor.format =
            bevy::render::render_resource::TextureFormat::Rgba32Float;
        result_image.texture_descriptor.usage |=
            bevy::render::render_resource::TextureUsages::STORAGE_BINDING;

        // Ensure result texture uses non-filtering sampler.
        result_image.sampler =
            bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            });

        // Create depth texture
        let mut depth_image = Image::new_uninit(
            bevy::render::render_resource::Extent3d {
                width: 2048,
                height: 2048,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            bevy::render::render_resource::TextureFormat::R32Float,
            bevy::asset::RenderAssetUsages::RENDER_WORLD,
        );
        depth_image.texture_descriptor.usage =
            bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::STORAGE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_SRC
                | bevy::render::render_resource::TextureUsages::COPY_DST;

        // Add sampler configuration to depth texture.
        depth_image.sampler =
            bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            });

        assets.result_texture = images.add(result_image);
        assets.depth_texture = images.add(depth_image);

        println!("✓ Compute-ready textures created with proper formats");
        loading_progress.textures_configured = true;
    }
}

// Abstracted texture configuration function
fn configure_texture_sampling(images: &mut ResMut<Assets<Image>>, assets: &PointCloudAssets) {
    use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

    let sampler_config = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..default()
    });

    for texture_handle in [
        &assets.position_texture,
        &assets.colour_class_texture,
        &assets.spatial_index_texture,
        &assets.result_texture,
        &assets.depth_texture,
    ] {
        if let Some(image) = images.get_mut(texture_handle) {
            image.sampler = sampler_config.clone();
        }
    }
}
