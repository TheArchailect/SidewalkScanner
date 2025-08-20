use crate::RenderModeState;
/// GPU-accelerated polygon classification compute pipeline (MVP)
use crate::engine::point_cloud::{PointCloudAssets, PointCloudBounds};
use crate::engine::render_mode::RenderMode;
use crate::tools::class_selection::ClassSelectionState;
use crate::tools::polygon::{ClassificationPolygon, PolygonClassificationData};
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

pub struct ComputeClassificationPlugin;

impl Plugin for ComputeClassificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComputeClassificationState>()
            .add_systems(Update, trigger_classification_compute);

        // Update the plugin:
        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app
                .init_resource::<ComputeClassificationState>()
                .init_resource::<PolygonClassificationData>()
                .init_resource::<PointCloudAssets>()
                .init_resource::<RenderModeState>()
                .init_resource::<ClassSelectionState>()
                .add_systems(bevy::render::Render, run_classification_compute);
        }
        app.add_plugins(ExtractResourcePlugin::<ComputeClassificationState>::default())
            .add_plugins(ExtractResourcePlugin::<PolygonClassificationData>::default())
            .add_plugins(ExtractResourcePlugin::<PointCloudAssets>::default())
            .add_plugins(ExtractResourcePlugin::<ClassSelectionState>::default())
            .add_plugins(ExtractResourcePlugin::<RenderModeState>::default());
    }
}

#[derive(Resource, Default, ExtractResource, Clone)]
pub struct ComputeClassificationState {
    pub should_recompute: bool,
    pub pipeline: Option<CachedComputePipelineId>,
    pub bind_group_layout: Option<BindGroupLayout>,
}

pub fn trigger_classification_compute(
    mut state: ResMut<ComputeClassificationState>,
    classification_data: Res<PolygonClassificationData>,
    render_mode: Res<RenderModeState>,
) {
    if classification_data.is_changed() || render_mode.is_changed() {
        state.should_recompute = true;
    }
}

pub fn run_classification_compute(
    mut state: ResMut<ComputeClassificationState>,
    classification_data: Res<PolygonClassificationData>,
    selection_state: Res<ClassSelectionState>,
    render_mode: Res<RenderModeState>,
    render_device: Res<RenderDevice>,
    mut render_queue: ResMut<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    mut gpu_images: ResMut<RenderAssets<GpuImage>>,
    assets: Res<PointCloudAssets>,
    asset_server: Res<AssetServer>,
) {
    let should_update = classification_data.is_changed()
        || render_mode.is_changed()
        || state.should_recompute
        || selection_state.is_changed();

    if !should_update {
        return;
    }

    if state.bind_group_layout.is_none() {
        initialize_compute_pipeline(&mut state, &render_device, &pipeline_cache, &asset_server);
    }

    let Some(bind_group_layout) = &state.bind_group_layout else {
        return;
    };
    let Some(pipeline_id) = state.pipeline else {
        return;
    };
    let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) else {
        return;
    };

    let Some(original_gpu) = gpu_images.get(&assets.colour_class_texture) else {
        return;
    };
    let Some(position_gpu) = gpu_images.get(&assets.position_texture) else {
        return;
    };
    let Some(spatial_gpu) = gpu_images.get(&assets.spatial_index_texture) else {
        return;
    };

    let Some(final_gpu) = gpu_images.get(&assets.final_texture) else {
        return;
    };

    execute_compute_shader(
        &render_device,
        &mut render_queue,
        pipeline,
        bind_group_layout,
        original_gpu,
        position_gpu,
        spatial_gpu,
        final_gpu,
        &classification_data.polygons,
        &selection_state,
        &assets.bounds,
        render_mode.current_mode,
    );

    state.should_recompute = false;
    println!(
        "GPU classification updated: {} polygons, mode: {:?}",
        classification_data.polygons.len(),
        render_mode.current_mode
    );
}

fn initialize_compute_pipeline(
    state: &mut ComputeClassificationState,
    render_device: &RenderDevice,
    pipeline_cache: &PipelineCache,
    asset_server: &AssetServer,
) {
    let bind_group_layout = render_device.create_bind_group_layout(
        "classification_compute_layout",
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
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba32Float,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 5,
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

    let shader = asset_server.load("shaders/modified_classification.wgsl");
    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("classification_compute".into()),
        layout: vec![bind_group_layout.clone()],
        push_constant_ranges: Vec::new(),
        shader,
        shader_defs: vec![],
        entry_point: "main".into(),
        zero_initialize_workgroup_memory: true,
    });

    state.bind_group_layout = Some(bind_group_layout);
    state.pipeline = Some(pipeline);
}

fn execute_compute_shader(
    render_device: &RenderDevice,
    render_queue: &mut RenderQueue,
    pipeline: &bevy::render::render_resource::ComputePipeline,
    bind_group_layout: &BindGroupLayout,
    original_gpu: &GpuImage,
    position_gpu: &GpuImage,
    spatial_gpu: &GpuImage,
    final_gpu: &GpuImage, // Use existing texture instead of creating new
    polygons: &[ClassificationPolygon],
    selection_state: &ClassSelectionState,
    bounds: &Option<PointCloudBounds>,
    current_mode: RenderMode,
) {
    let compute_buffer =
        create_compute_buffer(render_device, polygons, selection_state, current_mode);
    let bounds_buffer = create_bounds_buffer(render_device, bounds);

    let bind_group = render_device.create_bind_group(
        "classification_compute_bind_group",
        bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&original_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&position_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&spatial_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::TextureView(&final_gpu.texture_view), // Write to existing
            },
            BindGroupEntry {
                binding: 4,
                resource: compute_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 5,
                resource: bounds_buffer.as_entire_binding(),
            },
        ],
    );

    let mut encoder = render_device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("classification_compute"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(256, 256, 1);
    }
    render_queue.submit([encoder.finish()]);
}

fn create_compute_buffer(
    render_device: &RenderDevice,
    polygons: &[ClassificationPolygon],
    selection_state: &ClassSelectionState,
    current_mode: RenderMode,
) -> bevy::render::render_resource::Buffer {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct PolygonUniform {
        polygon_count: u32,
        total_points: u32,
        render_mode: u32,
        enable_spatial_opt: u32,
        selection_point: [f32; 2],
        is_selecting: u32,
        _padding: u32,
        point_data: [[f32; 4]; 512],
        polygon_info: [[f32; 4]; 64],
    }

    let mut uniform = PolygonUniform::zeroed();
    uniform.polygon_count = polygons.len().min(64) as u32;
    uniform.render_mode = current_mode as u32;
    uniform.enable_spatial_opt = 1;

    uniform.selection_point = selection_state
        .selection_point
        .map(|p| [p.x, p.y])
        .unwrap_or([0.0, 0.0]);
    uniform.is_selecting = if selection_state.is_selecting { 1 } else { 0 };

    let mut point_offset = 0;
    for (i, polygon) in polygons.iter().take(64).enumerate() {
        uniform.polygon_info[i] = [
            point_offset as f32,
            polygon.points.len() as f32,
            polygon.new_class as f32,
            0.0,
        ];

        for point in &polygon.points {
            if point_offset < 512 {
                uniform.point_data[point_offset] = [point.x, point.z, 0.0, 0.0];
                point_offset += 1;
            }
        }
    }

    uniform.total_points = point_offset as u32;

    render_device.create_buffer_with_data(&bevy::render::render_resource::BufferInitDescriptor {
        label: Some("polygon_data"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: BufferUsages::UNIFORM,
    })
}

fn create_bounds_buffer(
    render_device: &RenderDevice,
    bounds: &Option<PointCloudBounds>,
) -> bevy::render::render_resource::Buffer {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct BoundsUniform {
        min_bounds: [f32; 3],
        _padding1: f32,
        max_bounds: [f32; 3],
        _padding2: f32,
    }

    let uniform = if let Some(bounds) = bounds {
        BoundsUniform {
            min_bounds: [
                bounds.min_x() as f32,
                bounds.min_y() as f32,
                bounds.min_z() as f32,
            ],
            _padding1: 0.0,
            max_bounds: [
                bounds.max_x() as f32,
                bounds.max_y() as f32,
                bounds.max_z() as f32,
            ],
            _padding2: 0.0,
        }
    } else {
        BoundsUniform::zeroed()
    };

    render_device.create_buffer_with_data(&bevy::render::render_resource::BufferInitDescriptor {
        label: Some("bounds_data"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: BufferUsages::UNIFORM,
    })
}
