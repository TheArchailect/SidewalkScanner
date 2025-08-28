use crate::engine::point_cloud::PointCloudAssets;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::{
    render_asset::RenderAssets,
    render_resource::{
        BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
        BufferBindingType, BufferUsages, CachedComputePipelineId, ComputePassDescriptor,
        ComputePipelineDescriptor, PipelineCache, ShaderStages, StorageTextureAccess,
        TextureFormat, TextureSampleType, TextureViewDimension,
    },
    renderer::{RenderDevice, RenderQueue},
    texture::GpuImage,
};

pub struct EDLComputePlugin;

impl Plugin for EDLComputePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EDLComputeState>()
            .add_systems(Update, trigger_edl_compute)
            .add_plugins(ExtractResourcePlugin::<EDLComputeState>::default());

        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app.init_resource::<EDLRenderState>();
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct EDLComputeState {
    pub should_recompute: bool,
    pub camera_position: Vec3,
    pub camera_transform: Mat4,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

#[derive(Resource, Default)]
pub struct EDLRenderState {
    pub pipeline: Option<CachedComputePipelineId>,
    pub bind_group_layout: Option<BindGroupLayout>,
    pub initialized: bool,
}

impl EDLComputeState {
    pub fn new() -> Self {
        Self {
            should_recompute: false,
            camera_position: Vec3::default(),
            camera_transform: Mat4::default(),
            bounds_min: Vec3::default(),
            bounds_max: Vec3::default(),
        }
    }
}

impl ExtractResource for EDLComputeState {
    type Source = EDLComputeState;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

pub fn trigger_edl_compute(
    mut state: ResMut<EDLComputeState>,
    camera_query: Query<&GlobalTransform, (With<Camera3d>, Changed<GlobalTransform>)>,
    assets: Res<PointCloudAssets>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        state.should_recompute = true;
        state.camera_position = camera_transform.translation();
        state.camera_transform = camera_transform.compute_matrix();

        // Get bounds from assets
        if let Some(bounds) = &assets.bounds {
            state.bounds_min = Vec3::new(
                bounds.min_x() as f32,
                bounds.min_y() as f32,
                bounds.min_z() as f32,
            );
            state.bounds_max = Vec3::new(
                bounds.max_x() as f32,
                bounds.max_y() as f32,
                bounds.max_z() as f32,
            );
        }
    }
}

pub fn run_edl_compute(
    edl_state: Res<EDLComputeState>,
    mut render_state: ResMut<EDLRenderState>,
    render_device: Res<RenderDevice>,
    mut render_queue: ResMut<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    assets: Res<PointCloudAssets>,
    asset_server: Res<AssetServer>,
) {
    // Only recompute if needed AND we have valid textures
    if !edl_state.should_recompute {
        return;
    }

    // Check that we have the required textures
    let Some(position_gpu) = gpu_images.get(&assets.position_texture) else {
        println!("EDL: FAIL - position_texture not found in gpu_images");
        return;
    };

    let Some(final_gpu) = gpu_images.get(&assets.result_texture) else {
        println!("EDL: FAIL - result_texture not found in gpu_images");
        return;
    };

    let Some(edl_gpu) = gpu_images.get(&assets.result_texture_depth_alpha) else {
        println!("EDL: FAIL - edl_texture not found in gpu_images");
        return;
    };

    // Initialize pipeline only once using persistent render state
    if !render_state.initialized {
        initialise_depth_pipeline(
            &mut render_state,
            &render_device,
            &pipeline_cache,
            &asset_server,
        );
        if render_state.bind_group_layout.is_some() && render_state.pipeline.is_some() {
            render_state.initialized = true;
        } else {
            println!(
                "EDL: Pipeline initialization FAILED - bind_group_layout: {:?}, pipeline: {:?}",
                render_state.bind_group_layout.is_some(),
                render_state.pipeline.is_some()
            );
            return;
        }
    }

    let Some(bind_group_layout) = &render_state.bind_group_layout else {
        println!("EDL: FAIL - bind_group_layout is None after initialization");
        return;
    };

    let Some(pipeline_id) = render_state.pipeline else {
        println!("EDL: FAIL - pipeline_id is None after initialization");
        return;
    };

    let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) else {
        println!(
            "EDL Compute Depth: Pipeline not ready in cache yet for ID: {:?}",
            pipeline_id
        );
        return;
    };

    execute_depth_compute(
        &render_device,
        &mut render_queue,
        pipeline,
        bind_group_layout,
        position_gpu,
        final_gpu,
        edl_gpu,
        &edl_state,
    );
}

fn initialise_depth_pipeline(
    render_state: &mut EDLRenderState,
    render_device: &RenderDevice,
    pipeline_cache: &PipelineCache,
    asset_server: &AssetServer,
) {
    let bind_group_layout = render_device.create_bind_group_layout(
        "edl_compute_layout",
        &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    );
    let shader = asset_server.load("shaders/compute_depth.wgsl");

    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("compute_depth".into()),
        layout: vec![bind_group_layout.clone()],
        push_constant_ranges: Vec::new(),
        shader,
        shader_defs: vec![],
        entry_point: "main".into(),
        zero_initialize_workgroup_memory: true,
    });

    render_state.bind_group_layout = Some(bind_group_layout);
    render_state.pipeline = Some(pipeline);
}

fn execute_depth_compute(
    render_device: &RenderDevice,
    render_queue: &mut RenderQueue,
    pipeline: &bevy::render::render_resource::ComputePipeline,
    bind_group_layout: &BindGroupLayout,
    position_gpu: &GpuImage,
    final_gpu: &GpuImage,
    edl_gpu: &GpuImage,
    state: &EDLComputeState,
) {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct EDLUniforms {
        view_matrix: [[f32; 4]; 4],
        camera_pos: [f32; 3],
        _padding1: f32,
        bounds_min: [f32; 3],
        _padding2: f32,
        bounds_max: [f32; 3],
        _padding3: f32,
    }

    let uniforms = EDLUniforms {
        view_matrix: state.camera_transform.to_cols_array_2d(),
        camera_pos: state.camera_position.to_array(),
        _padding1: 0.0,
        bounds_min: state.bounds_min.to_array(),
        _padding2: 0.0,
        bounds_max: state.bounds_max.to_array(),
        _padding3: 0.0,
    };

    let uniform_buffer = render_device.create_buffer_with_data(
        &bevy::render::render_resource::BufferInitDescriptor {
            label: Some("edl_uniform"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: BufferUsages::UNIFORM,
        },
    );

    let bind_group = render_device.create_bind_group(
        "edl_compute_bind_group",
        bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&position_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&final_gpu.texture_view), // INPUT
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&edl_gpu.texture_view), // OUTPUT
            },
            BindGroupEntry {
                binding: 3,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    );

    let workgroups_x = (final_gpu.size.width + 7) / 8;
    let workgroups_y = (final_gpu.size.height + 7) / 8;

    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("compute_depth"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroups_x, workgroups_y, 1);
    }
    render_queue.submit([encoder.finish()]);
}
