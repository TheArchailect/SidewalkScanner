/// Unified texture resolution for all generated textures (positions, colour+class, heightmap)
pub const TEXTURE_SIZE: usize = 2048;

/// Maximum points that can fit in a texture
pub const MAX_POINTS: usize = TEXTURE_SIZE * TEXTURE_SIZE;

/// Heightmap blend radius for road surface smoothing (pixels)
pub const HEIGHTMAP_BLEND_RADIUS: f32 = 64.0;

/// Sample size for colour detection
pub const COLOUR_DETECTION_SAMPLE_SIZE: usize = 100;
