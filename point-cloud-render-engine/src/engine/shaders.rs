use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    OriginalClassification = 0,
    ModifiedClassification = 1,
    RgbColour = 2,
    MortonCode = 3,
    PerformanceDebug = 4,
}

/// Simplified point cloud shader material - only textures needed for rendering
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PointCloudShader {
    #[texture(0)]
    #[sampler(1)]
    pub position_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub final_texture: Handle<Image>, // Output from compute shader

    #[uniform(4)]
    pub params: [Vec4; 2], // [min_bounds + texture_size, max_bounds]
}

impl Material for PointCloudShader {
    fn vertex_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }
}

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
