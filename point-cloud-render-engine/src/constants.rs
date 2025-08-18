/// Shared configuration for unified texture processing

/// Unified texture resolution for all generated textures (positions, colour+class, heightmap)
pub const TEXTURE_SIZE: usize = 2048;

/// Maximum points that can fit in a texture
pub const MAX_POINTS: usize = TEXTURE_SIZE * TEXTURE_SIZE;

/// Road classification codes for heightmap generation
pub const ROAD_CLASSIFICATIONS: &[u8] = &[2, 10, 11, 12];

/// Heightmap blend radius for road surface smoothing (pixels)
pub const HEIGHTMAP_BLEND_RADIUS: f32 = 16.0;

/// Sample size for colour detection during preprocessing
pub const COLOUR_DETECTION_SAMPLE_SIZE: usize = 100;

/// Coordinate transformation matrix (row-major: [x_new, y_new, z_new])
/// Default: -90° X rotation (Z→Y, -Y→Z, X→X)
pub const COORDINATE_TRANSFORM: [[f64; 3]; 3] = [
    [1.0, 0.0, 0.0],  // X = X
    [0.0, 0.0, 1.0],  // Y = Z
    [0.0, -1.0, 0.0], // Z = -Y
];
