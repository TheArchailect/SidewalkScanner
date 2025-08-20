/// Unified texture point cloud shader material
use bevy::render::render_resource::ShaderType;
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
}

#[derive(Debug, Clone, Copy, ShaderType)]
#[repr(C)]
pub struct PolygonClassificationUniform {
    pub polygon_count: u32,
    pub total_points: u32,
    pub render_mode: u32,
    pub _padding: u32,
    pub point_data: [Vec4; 512], // [x, z, 0, 0]
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

/// Point cloud shader material with unified texture bindings
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PointCloudShader {
    #[texture(0)]
    #[sampler(1)]
    pub position_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub colour_class_texture: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub spatial_index_texture: Handle<Image>,

    #[uniform(6)]
    pub params: [Vec4; 2],

    #[uniform(7)]
    pub polygon_data: PolygonClassificationUniform,
}

impl Material for PointCloudShader {
    fn vertex_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }
}
