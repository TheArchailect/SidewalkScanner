use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PointCloudShader {
    #[texture(0)]
    #[sampler(1)]
    pub position_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub metadata_texture: Handle<Image>,

    #[uniform(4)]
    pub params: [Vec4; 2],
}

impl Material for PointCloudShader {
    fn vertex_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "./shaders/point_cloud.wgsl".into()
    }
}
