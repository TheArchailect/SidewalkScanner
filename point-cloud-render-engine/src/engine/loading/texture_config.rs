use bevy::prelude::*;

use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::loading::progress::LoadingProgress;
use constants::texture::TEXTURE_SIZE;

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
                width: TEXTURE_SIZE as u32,
                height: TEXTURE_SIZE as u32,
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
