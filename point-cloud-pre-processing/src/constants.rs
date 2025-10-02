/// Shared configuration for point cloud processing

/// Unified texture resolution for all generated textures
pub const TEXTURE_SIZE: usize = 2048;

/// Maximum points that can fit in a texture
pub const MAX_POINTS: usize = TEXTURE_SIZE * TEXTURE_SIZE;

/// Road classification codes for heightmap generation
pub const ROAD_CLASSIFICATIONS: &[u8] = &[2, 10, 11, 12];

/// Heightmap blend radius for road surface smoothing (pixels)
pub const HEIGHTMAP_BLEND_RADIUS: f32 = 64.0;

/// Sample size for colour detection
pub const COLOUR_DETECTION_SAMPLE_SIZE: usize = 100;

/// Coordinate transformation matrix (row-major: [x_new, y_new, z_new])
/// Default: -90° X rotation (Z→Y, -Y→Z, X→X)
pub const COORDINATE_TRANSFORM: [[f64; 3]; 3] = [
    [1.0, 0.0, 0.0],  // X = X
    [0.0, 0.0, 1.0],  // Y = Z
    [0.0, -1.0, 0.0], // Z = -Y
];

pub struct ClassInfo {
    pub id: u8,
    pub name: &'static str,
}

pub const CLASS_MAP: &[ClassInfo] = &[
    ClassInfo {
        id: 0,
        name: "unclassified",
    },
    ClassInfo {
        id: 2,
        name: "ground, sidewalk",
    },
    ClassInfo {
        id: 3,
        name: "vegetation - low",
    },
    ClassInfo {
        id: 4,
        name: "vegetation - medium",
    },
    ClassInfo {
        id: 5,
        name: "vegetation - high",
    },
    ClassInfo {
        id: 6,
        name: "buildings",
    },
    ClassInfo {
        id: 8,
        name: "street furniture",
    },
    ClassInfo {
        id: 11,
        name: "street pavement",
    },
    ClassInfo {
        id: 15,
        name: "cars, trucks",
    },
];

pub fn get_class_name(id: u8) -> String {
    CLASS_MAP
        .iter()
        .find(|c| c.id == id)
        .map_or("unknown", |c| c.name)
        .to_string()
}
