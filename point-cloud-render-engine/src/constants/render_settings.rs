use crate::engine::render::edl_post_processing::EDLSettings;

pub const EDL_SETTINGS: EDLSettings = EDLSettings {
    radius: 3.0,
    strength: 50.0,
    ambient_boost: 0.6,
    contrast: 1.1,
};

pub const DRAW_LINE_WIDTH: f32 = 0.076;
pub const MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE: f32 = 0.125;
pub const DRAW_VERTEX_SIZE: f32 = 0.08;
