use crate::constants::texture::TEXTURE_SIZE;
/// Heightmap sampling utilities for terrain intersection
use crate::engine::assets::bounds::PointCloudBounds;

use bevy::prelude::*;

/// Sample heightmap at normalised coordinates with bilinear interpolation
pub fn sample_heightmap_bilinear(
    heightmap_image: &Image,
    norm_x: f32,
    norm_z: f32,
    bounds: &PointCloudBounds,
) -> f32 {
    let data = heightmap_image
        .data
        .as_ref()
        .expect("Heightmap image data not available");

    // Convert normalized coords to continuous pixel space
    let pixel_x_f = norm_x * (TEXTURE_SIZE - 1) as f32;
    let pixel_z_f = norm_z * (TEXTURE_SIZE - 1) as f32;

    // Get integer pixel coordinates
    let x0 = pixel_x_f.floor() as usize;
    let z0 = pixel_z_f.floor() as usize;
    let x1 = (x0 + 1).min(TEXTURE_SIZE - 1);
    let z1 = (z0 + 1).min(TEXTURE_SIZE - 1);

    // Calculate interpolation weights
    let wx = pixel_x_f - x0 as f32;
    let wz = pixel_z_f - z0 as f32;

    // Sample four corners
    let h00 = sample_height_at_pixel(data, x0, z0);
    let h10 = sample_height_at_pixel(data, x1, z0);
    let h01 = sample_height_at_pixel(data, x0, z1);
    let h11 = sample_height_at_pixel(data, x1, z1);

    // Bilinear interpolation
    let h_top = h00 * (1.0 - wx) + h10 * wx;
    let h_bottom = h01 * (1.0 - wx) + h11 * wx;
    let normalized_height = h_top * (1.0 - wz) + h_bottom * wz;

    // Denormalise to world coordinates
    bounds.min_y() as f32 + normalized_height * (bounds.max_y() - bounds.min_y()) as f32
}

/// Sample height at specific pixel coordinates
fn sample_height_at_pixel(data: &[u8], x: usize, z: usize) -> f32 {
    // Check if coordinates are within texture bounds
    if x >= TEXTURE_SIZE || z >= TEXTURE_SIZE {
        return 0.0; // Return default height for out-of-bounds access
    }

    let pixel_index = (z * TEXTURE_SIZE + x) * 4; // 4 bytes per f32

    // Check if we have enough data to read
    if pixel_index + 4 > data.len() {
        return 0.0; // Return default height if data is insufficient
    }

    let height_bytes = &data[pixel_index..pixel_index + 4];
    f32::from_le_bytes([
        height_bytes[0],
        height_bytes[1],
        height_bytes[2],
        height_bytes[3],
    ])
}
