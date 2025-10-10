use crate::constants::procedural_shader::{
    MAX_IGNORE_MASK_LENGTH, MAXIMUM_POLYGON_POINTS, MAXIMUM_POLYGONS,
};
use crate::engine::assets::bounds::BoundsData;
use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::assets::scene_manifest::SceneManifest;
use crate::engine::systems::render_mode::RenderMode;
use crate::engine::systems::render_mode::{MouseEnterObjectState, RenderModeState};
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

/// Runtime state for controlling classification compute pipeline execution.
///
/// This resource tracks:
/// - Whether a recomputation should occur (`should_recompute`)
/// - Cached pipeline ID and bind group layout
///
/// The state is initialised by the plugin and shared across systems that
/// prepare or trigger compute dispatches.
///
/// Used by the render extract phase to synchronise data to the GPU side.
#[derive(Resource, Default, ExtractResource, Clone)]
pub struct ComputeClassificationState {
    pub should_recompute: bool,
    pub pipeline: Option<CachedComputePipelineId>,
    pub bind_group_layout: Option<BindGroupLayout>,
}

impl Plugin for ComputeClassificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComputeClassificationState>()
            .add_systems(Update, trigger_classification_compute)
            .add_plugins(ExtractResourcePlugin::<ComputeClassificationState>::default())
            .add_plugins(ExtractResourcePlugin::<PolygonClassificationData>::default())
            .add_plugins(ExtractResourcePlugin::<ClassSelectionState>::default());
    }
}

/// System that flags when the classification compute pipeline should re-run.
///
/// It checks for changes in:
/// - Polygon classification data
/// - Render mode (RGB | Original | Modified | Connectivity)
/// - Mouse hover object selection for use feedback
///
/// If any of these change, `should_recompute` is set to `true` in the shared state.
///
/// This helps defer compute shader execution until it's required, improving efficiency.
pub fn trigger_classification_compute(
    mut state: ResMut<ComputeClassificationState>,
    classification_data: Res<PolygonClassificationData>,
    render_mode: Res<RenderModeState>,
    mouse_enter_object_id: Res<MouseEnterObjectState>,
) {
    if classification_data.is_changed()
        || render_mode.is_changed()
        || mouse_enter_object_id.is_changed()
    {
        state.should_recompute = true;
    }
}

/// Executes the classification compute shader when relevant state has changed.
///
/// This function:
/// - Lazily initialises the compute pipeline if needed
/// - Retrieves necessary GPU textures
/// - Creates uniform buffers for polygons and terrain bounds
/// - Dispatches the WGSL compute shader
///
/// ### WGSL expectations:
/// The WGSL shader bound at `shaders/modified_classification.wgsl` expects:
/// - 3 input textures: colour class, position, and spatial index
/// - 1 writable output texture
/// - 2 uniform buffers:
///   - Polygon + mask metadata
///   - Scene bounds
///
/// These are mapped to bindings 0–5 in the WGSL:
/// ```wgsl
/// @group(0) @binding(0) var colour_texture: texture_2d<f32>;
/// @group(0) @binding(1) var position_texture: texture_2d<f32>;
/// @group(0) @binding(2) var spatial_index_texture: texture_2d<f32>;
/// @group(0) @binding(3) var result_texture: texture_storage_2d<rgba32float, write>;
/// @group(0) @binding(4) var<uniform> polygon_data: PolygonUniform;
/// @group(0) @binding(5) var<uniform> terrain_bounds: TerrainBounds;
/// ```
pub fn run_classification_compute(
    mut state: ResMut<ComputeClassificationState>,
    classification_data: Res<PolygonClassificationData>,
    selection_state: Res<ClassSelectionState>,
    render_mode: Res<RenderModeState>,
    mouse_enter_object_id: Res<MouseEnterObjectState>,
    render_device: Res<RenderDevice>,
    mut render_queue: ResMut<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
    gpu_images: ResMut<RenderAssets<GpuImage>>,
    assets: Res<PointCloudAssets>,
    asset_server: Res<AssetServer>,
    manifest: Res<SceneManifest>,
) {
    let should_update = classification_data.is_changed()
        || render_mode.is_changed()
        || mouse_enter_object_id.is_changed()
        || state.should_recompute
        || selection_state.is_changed();

    if !should_update {
        return;
    }

    if state.bind_group_layout.is_none() {
        initialise_compute_pipeline(&mut state, &render_device, &pipeline_cache, &asset_server);
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

    let Some(final_gpu) = gpu_images.get(&assets.result_texture) else {
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
        manifest.terrain_bounds(), // Pass terrain bounds directly from manifest.
        render_mode.current_mode,
        mouse_enter_object_id.object_id,
    );

    state.should_recompute = false;
}

/// Creates the compute pipeline and bind group layout used for classification.
///
/// Sets up the following bindings:
/// 0–2: Input textures (read-only)
/// 3:   Output texture (write-only)
/// 4–5: Uniform buffers (polygon data + terrain bounds)
///
/// Expects the shader to be located at `shaders/modified_classification.wgsl`.
fn initialise_compute_pipeline(
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
    final_gpu: &GpuImage,
    polygons: &[ClassificationPolygon],
    selection_state: &ClassSelectionState,
    terrain_bounds: &BoundsData, // Accept terrain bounds directly.
    current_mode: RenderMode,
    mouse_enter_object_id: Option<u32>,
) {
    let compute_buffer = create_compute_buffer(
        render_device,
        polygons,
        selection_state,
        current_mode,
        mouse_enter_object_id,
    );
    let bounds_buffer = create_terrain_bounds_buffer(render_device, terrain_bounds);

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
                resource: BindingResource::TextureView(&final_gpu.texture_view),
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

/// encode a polygon tool operation ```polygon | class | object_id | mode```
/// this will be decoded on the gpu to perform procedural non-destructive modifications
fn encode_mask(poly_idx: u32, mask_id0: u32, mask_id1: u32, mode: u32) -> u32 {
    assert!(
        poly_idx < MAXIMUM_POLYGONS as u32,
        "poly_idx must be < MAXIMUM_POLYGONS"
    );
    assert!(
        mask_id0 < MAX_IGNORE_MASK_LENGTH as u32,
        "mask_id.0 must be < MAX_IGNORE_MASK_LENGTH"
    );
    assert!(
        mask_id1 < MAX_IGNORE_MASK_LENGTH as u32,
        "mask_id.1 must be < MAX_IGNORE_MASK_LENGTH"
    );
    assert!(mode <= 1, "mode must be 0 or 1");

    let encoded: u32 = (mask_id0 & 0x1FF) |              // bits 0–8
        ((mask_id1 & 0x1FF) << 9) |       // bits 9–17
        ((poly_idx & 0x1FF) << 18) |      // bits 18–26
        ((mode & 0x1) << 27); // bit 27

    encoded
}

fn create_compute_buffer(
    render_device: &RenderDevice,
    polygons: &[ClassificationPolygon],
    selection_state: &ClassSelectionState,
    current_mode: RenderMode,
    mouse_enter_object_id: Option<u32>,
) -> bevy::render::render_resource::Buffer {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Clone, Copy, Pod, Zeroable)]
    pub struct ComputeUniformData {
        pub polygon_count: u32,      // 0
        pub total_points: u32,       // 4
        pub render_mode: u32,        // 8
        pub enable_spatial_opt: u32, // 12

        pub selection_point: [f32; 4], // 16 (vec3 + pad)
        pub is_selecting: u32,         // 32
        pub hover_object_id: u32,      // 36
        pub _padding: [u32; 2],        // 40 (8 bytes → next @48)

        pub point_data: [[f32; 4]; MAXIMUM_POLYGON_POINTS], // 48, stride 16
        pub polygon_info: [[f32; 4]; MAXIMUM_POLYGONS],     // after that
        pub ignore_masks: [[u32; 4]; MAX_IGNORE_MASK_LENGTH],
    }

    let mut uniform = ComputeUniformData::zeroed();

    if let Some(id) = mouse_enter_object_id {
        info!("Recieved updated hover ID at compute shader: {:?}", id);
        uniform.hover_object_id = id as u32;
    }

    uniform.polygon_count = polygons.len() as u32;
    uniform.render_mode = current_mode as u32;
    uniform.enable_spatial_opt = 1;

    // Encode our mask id's along with the polygon index and polygon mode (hide or reclassify)
    // so we can decode these complicated operations and relationships on the GPU in a per-point context
    let mut mask_offset = 0;
    for (poly_idx, polygon) in polygons.iter().enumerate().take(MAXIMUM_POLYGONS) {
        for (mask_idx, &(mask_id0, mask_id1)) in polygon.masks.iter().enumerate() {
            let encoded = encode_mask(
                poly_idx as u32,
                mask_id0,
                mask_id1,
                polygon.mode.clone() as u32,
            );

            let i = mask_offset + mask_idx;
            assert!(
                (i as usize) < MAX_IGNORE_MASK_LENGTH * 4,
                "mask array overflow: i={} capacity={}",
                i,
                MAX_IGNORE_MASK_LENGTH * 4
            );
            uniform.ignore_masks[i / 4][i % 4] = encoded;
        }
        mask_offset += polygon.masks.len();
    }

    uniform.selection_point = selection_state
        .selection_point
        .map(|p| [p.x, p.y, p.z, 0.0])
        .unwrap_or([0.0, 0.0, 0.0, 0.0]);

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
            if point_offset < MAXIMUM_POLYGON_POINTS {
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

/// Create uniform buffer from terrain bounds data without legacy conversion.
fn create_terrain_bounds_buffer(
    render_device: &RenderDevice,
    terrain_bounds: &BoundsData,
) -> bevy::render::render_resource::Buffer {
    use bytemuck::{Pod, Zeroable};

    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct TerrainBoundsUniform {
        min_bounds: [f32; 3],
        _padding1: f32,
        max_bounds: [f32; 3],
        _padding2: f32,
    }

    let uniform = TerrainBoundsUniform {
        min_bounds: [
            terrain_bounds.min_x as f32,
            terrain_bounds.min_y as f32,
            terrain_bounds.min_z as f32,
        ],
        _padding1: 0.0,
        max_bounds: [
            terrain_bounds.max_x as f32,
            terrain_bounds.max_y as f32,
            terrain_bounds.max_z as f32,
        ],
        _padding2: 0.0,
    };

    render_device.create_buffer_with_data(&bevy::render::render_resource::BufferInitDescriptor {
        label: Some("terrain_bounds_data"),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: BufferUsages::UNIFORM,
    })
}
