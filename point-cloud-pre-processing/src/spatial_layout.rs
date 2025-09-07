/// Simple Z-Order spatial layout for point cloud textures
use crate::bounds::PointCloudBounds;
use crate::constants::TEXTURE_SIZE;

/// Point with spatial metadata for texture generation
#[derive(Debug, Clone)]
pub struct SpatialPoint {
    pub world_pos: (f64, f64, f64),
    pub norm_pos: (f32, f32, f32),
    pub morton_index: u32,
    pub spatial_cell_id: u32,
    pub classification: u8,
    pub color: Option<(u16, u16, u16)>,
    pub object_number: f32,
}

/// Spatial texture generator with Z-order layout
pub struct SpatialTextureGenerator {
    pub bounds: PointCloudBounds,
    pub grid_resolution: u32,
    pub points: Vec<SpatialPoint>,
}

impl SpatialTextureGenerator {
    /// Create new spatial generator with Z-order encoding
    pub fn new(bounds: PointCloudBounds, grid_resolution: u32) -> Self {
        Self {
            bounds,
            grid_resolution,
            points: Vec::new(),
        }
    }

    /// Add point with spatial encoding
    pub fn add_point(
        &mut self,
        world_pos: (f64, f64, f64),
        classification: u8,
        color: Option<(u16, u16, u16)>,
        object_number: f32,
    ) {
        let norm_pos = (
            self.bounds.normalize_x(world_pos.0),
            self.bounds.normalize_y(world_pos.1),
            self.bounds.normalize_z(world_pos.2),
        );

        // Calculate spatial grid coordinates
        let grid_x = (norm_pos.0 * (self.grid_resolution - 1) as f32)
            .clamp(0.0, (self.grid_resolution - 1) as f32) as u32;
        let grid_z = (norm_pos.2 * (self.grid_resolution - 1) as f32)
            .clamp(0.0, (self.grid_resolution - 1) as f32) as u32;

        // Generate Morton index for Z-order curve
        let morton_index = morton_encode_2d(grid_x, grid_z);
        let spatial_cell_id = grid_z * self.grid_resolution + grid_x;

        let spatial_point = SpatialPoint {
            world_pos,
            norm_pos,
            morton_index,
            spatial_cell_id,
            classification,
            color,
            object_number,
        };

        self.points.push(spatial_point);
    }

    /// Sort points by Morton index for spatial locality
    pub fn sort_spatially(&mut self) {
        self.points
            .sort_by(|a, b| a.morton_index.cmp(&b.morton_index));
    }

    /// Generate spatial index texture with split Morton codes
    pub fn generate_spatial_index_texture(&self) -> Vec<f32> {
        let mut spatial_data = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE * 2]; // RG format

        for (i, point) in self.points.iter().enumerate() {
            let base_idx = i * 2;
            if base_idx + 3 < spatial_data.len() {
                // Store as reinterpreted floats (preserves all 32 bits)
                spatial_data[base_idx] = f32::from_bits(point.morton_index); // R: code 32 bits
                spatial_data[base_idx + 1] = point.spatial_cell_id as f32; // G: cell ID 32 bits
                // spatial_data[base_idx + 2] = point.spatial_cell_id as f32; // B: cell ID
                // spatial_data[base_idx + 3] = i as f32; // A: point index
            }
        }

        // To reconstruct 64-bit Morton from RG channels on the GPU
        // uint morton_high = floatBitsToUint(texelFetch(spatialTexture, coord, 0).r);
        // uint morton_low = floatBitsToUint(texelFetch(spatialTexture, coord, 0).g);
        // uint64_t full_morton = (uint64_t(morton_high) << 32) | morton_low;

        spatial_data
    }

    /// Generate position texture with spatial ordering
    pub fn generate_position_texture(&self) -> Vec<f32> {
        let mut position_data = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE * 4];

        for (i, point) in self.points.iter().enumerate() {
            let base_idx = i * 4;
            if base_idx + 3 < position_data.len() {
                position_data[base_idx] = point.norm_pos.0;
                position_data[base_idx + 1] = point.norm_pos.1;
                position_data[base_idx + 2] = point.norm_pos.2;
                position_data[base_idx + 3] = point.object_number / 121.0;
            }
        }

        position_data
    }

    /// Generate colour+classification texture with spatial ordering
    pub fn generate_colour_class_texture(&self) -> Vec<f32> {
        let mut colour_data = vec![0.0f32; TEXTURE_SIZE * TEXTURE_SIZE * 4];

        for (i, point) in self.points.iter().enumerate() {
            let base_idx = i * 4;
            if base_idx + 3 < colour_data.len() {
                if let Some((r, g, b)) = point.color {
                    colour_data[base_idx] = r as f32 / 65535.0;
                    colour_data[base_idx + 1] = g as f32 / 65535.0;
                    colour_data[base_idx + 2] = b as f32 / 65535.0;
                } else {
                    colour_data[base_idx] = 1.0;
                    colour_data[base_idx + 1] = 1.0;
                    colour_data[base_idx + 2] = 1.0;
                }
                colour_data[base_idx + 3] = point.classification as f32 / 255.0;
            }
        }

        colour_data
    }
}

// /// Morton encoding for 2D coordinates
// pub fn morton_encode_2d(x: u32, z: u32) -> u64 {
//     let mut result = 0u64;
//     for i in 0..16 {
//         // 16 bits is enough for up to 65536x65536 grid
//         result |= ((x & (1 << i)) as u64) << (2 * i); // Interleave X bits at even positions
//         result |= ((z & (1 << i)) as u64) << (2 * i + 1); // Interleave Z bits at odd positions
//     }
//     result
// }

/// Morton encoding for 2D coordinates (32-bit version)
/// This version can handle coordinates up to 65535x65535 (16 bits each)
pub fn morton_encode_2d(x: u32, z: u32) -> u32 {
    let mut result = 0u32;
    for i in 0..16 {
        // 16 bits each for x and z to fit in 32-bit result (32 bits total)
        result |= x & (1 << i) << i; // Interleave X bits at even positions
        result |= z & (1 << i) << (i + 1); // Interleave Z bits at odd positions
    }
    result
}
