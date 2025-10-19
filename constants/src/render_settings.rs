use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;

#[derive(
    Component, Default, Clone, Copy, ExtractComponent, bevy::render::render_resource::ShaderType,
)]
pub struct EDLSettings {
    pub radius: f32,
    pub strength: f32,
    pub ambient_boost: f32,
    pub contrast: f32,
}

pub const EDL_SETTINGS: EDLSettings = EDLSettings {
    radius: 3.0,
    strength: 50.0,
    ambient_boost: 0.6,
    contrast: 1.1,
};

pub const DRAW_LINE_WIDTH: f32 = 0.076;
pub const MOUSE_RAYCAST_INTERSECTION_SPHERE_SIZE: f32 = 0.125;
pub const DRAW_VERTEX_SIZE: f32 = 0.08;
