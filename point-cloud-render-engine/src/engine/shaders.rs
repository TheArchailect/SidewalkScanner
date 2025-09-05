use crate::engine::render_mode::RenderMode;
use bevy::prelude::*;

/// Uniform data structure for polygon classification compute shaders.
/// Maintains compatibility with existing compute pipeline while removing
/// dependency on Bevy's Material trait system.
#[derive(Debug, Clone, Copy, bevy::render::render_resource::ShaderType)]
#[repr(C)]
pub struct PolygonClassificationUniform {
    pub polygon_count: u32,
    pub total_points: u32,
    pub render_mode: u32,
    pub _padding: u32,
    pub point_data: [Vec4; 512],
    pub polygon_info: [Vec4; 64],
}

impl Default for PolygonClassificationUniform {
    fn default() -> Self {
        Self {
            polygon_count: 0,
            total_points: 0,
            render_mode: RenderMode::RgbColour as u32,
            _padding: 0,
            point_data: [Vec4::ZERO; 512],
            polygon_info: [Vec4::ZERO; 64],
        }
    }
}
