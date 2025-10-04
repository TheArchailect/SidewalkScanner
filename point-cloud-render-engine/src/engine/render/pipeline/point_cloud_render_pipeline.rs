use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::core::app_state::{AppState, PipelineDebugState};
use crate::engine::render::extraction::render_state::extract_point_cloud_render_state;
use crate::engine::systems::render_mode::RenderModeState;
use bevy::core_pipeline::core_3d::graph::{Core3d, Node3d};
use bevy::ecs::{
    query::QueryItem,
    system::{SystemParamItem, lifetimeless::SRes},
};
use bevy::math::FloatOrd;
use bevy::pbr::{
    DrawMesh, MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey, RenderMeshInstances,
    SetMeshBindGroup, SetMeshViewBindGroup,
};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::mesh::{MeshVertexBufferLayoutRef, RenderMesh};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
};
use bevy::render::render_phase::{
    AddRenderCommand, CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, PhaseItem,
    PhaseItemExtraIndex, SetItemPipeline, SortedPhaseItem, SortedRenderPhasePlugin,
    ViewSortedRenderPhases,
};
use bevy::render::render_resource::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
    Buffer, BufferBindingType, BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites,
    FragmentState, MultisampleState, PipelineCache, PrimitiveState, RenderPassDescriptor,
    RenderPipelineDescriptor, ShaderStages, SpecializedMeshPipeline, SpecializedMeshPipelineError,
    SpecializedMeshPipelines, TextureFormat, TextureSampleType, TextureViewDimension, VertexState,
};
use bevy::render::{
    Extract, Render, RenderApp, RenderDebugFlags, RenderSet,
    render_asset::RenderAssets,
    renderer::{RenderContext, RenderDevice},
    sync_world::MainEntity,
    texture::GpuImage,
    view::{ExtractedView, RenderVisibleEntities, ViewTarget},
};
use std::ops::Range;

pub struct PointCloudRenderPlugin;

pub fn debug_view_extraction(cameras: Extract<Query<Entity, With<Camera3d>>>) {
    let camera_count = cameras.iter().count();
    if camera_count == 0 {
        warn!("No Camera3d entities found during extraction!");
    }
}

pub fn debug_phase_creation(phases: Res<ViewSortedRenderPhases<PointCloudPhase>>) {
    if phases.is_empty() {
        warn!("No PointCloudPhase instances created!");
    }
}

impl Plugin for PointCloudRenderPlugin {
    fn build(&self, app: &mut App) {
        // Add extraction plugins for resources needed in render world.
        app.add_plugins((
            ExtractResourcePlugin::<PointCloudRenderState>::default(),
            ExtractResourcePlugin::<PointCloudAssets>::default(),
            ExtractResourcePlugin::<RenderModeState>::default(),
            SortedRenderPhasePlugin::<PointCloudPhase, MeshPipeline>::new(
                RenderDebugFlags::default(),
            ),
        ));

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<SpecializedMeshPipelines<PointCloudPipeline>>()
            .init_resource::<DrawFunctions<PointCloudPhase>>()
            .add_render_command::<PointCloudPhase, DrawPointCloud>()
            .init_resource::<ViewSortedRenderPhases<PointCloudPhase>>()
            .init_resource::<PreparedPointCloudBindGroups>()
            .add_systems(
                bevy::render::ExtractSchedule,
                extract_point_cloud_render_state,
            )
            .add_systems(
                Render,
                (
                    prepare_point_cloud_bind_groups.in_set(RenderSet::PrepareBindGroups),
                    queue_point_cloud_meshes.in_set(RenderSet::QueueMeshes),
                    bevy::render::render_phase::sort_phase_system::<PointCloudPhase>
                        .in_set(RenderSet::PhaseSort),
                )
                    .run_if(in_state(AppState::Running)),
            )
            .add_systems(ExtractSchedule, debug_view_extraction)
            .add_systems(Render, debug_phase_creation.in_set(RenderSet::PhaseSort))
            .add_render_graph_node::<ViewNodeRunner<PointCloudRenderNode>>(
                Core3d,
                PointCloudRenderLabel,
            )
            .add_render_graph_edges(
                Core3d,
                (
                    PointCloudRenderLabel,
                    Node3d::MainOpaquePass,
                    Node3d::Tonemapping,
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<PointCloudPipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct PointCloudRenderLabel;

/// Resource containing point cloud rendering parameters extracted from main world.
#[derive(Resource, Default, ExtractResource, Clone)]
pub struct PointCloudRenderState {
    pub camera_position: Vec3,
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
    pub texture_size: f32,
    pub should_render: bool,
}

/// Component marking entities for point cloud rendering pipeline.
#[derive(Component, Clone, ExtractComponent)]
pub struct PointCloudRenderable {
    pub point_count: u32,
}

/// Render pipeline resource managing specialized mesh pipeline for point clouds.
/// Integrates with Bevy's mesh rendering system for automatic view/mesh binding.
/// Caches the material bind group layout to avoid world access during specialization.
#[derive(Resource)]
pub struct PointCloudPipeline {
    mesh_pipeline: MeshPipeline,
    shader_handle: Handle<Shader>,
    /// Cached bind group layout for @group(2) material bindings.
    /// Created during FromWorld initialization to avoid runtime world access.
    material_bind_group_layout: BindGroupLayout,
}

impl FromWorld for PointCloudPipeline {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh_pipeline: MeshPipeline::from_world(world),
            shader_handle: world
                .resource::<AssetServer>()
                .load("shaders/point_cloud.wgsl"),
            // Pass render device directly without holding a reference.
            material_bind_group_layout: create_point_cloud_material_bind_group_layout(
                world.resource::<RenderDevice>(),
            ),
        }
    }
}

impl SpecializedMeshPipeline for PointCloudPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        // Point cloud uses minimal vertex attributes - position only.
        let mut vertex_attributes = Vec::new();
        if layout.0.contains(Mesh::ATTRIBUTE_POSITION) {
            vertex_attributes.push(Mesh::ATTRIBUTE_POSITION.at_shader_location(0));
        }

        let vertex_buffer_layout = layout.0.get_layout(&vertex_attributes)?;

        Ok(RenderPipelineDescriptor {
            label: Some("Point Cloud Render Pipeline".into()),
            layout: vec![
                // @group(0) - View uniforms (automatic binding via SetMeshViewBindGroup).
                self.mesh_pipeline
                    .get_view_layout(MeshPipelineViewLayoutKey::from(key))
                    .clone(),
                // @group(1) - Mesh uniforms (automatic binding via SetMeshBindGroup).
                self.mesh_pipeline.mesh_layouts.model_only.clone(),
                // @group(2) - Point cloud material uniforms (cached layout).
                self.material_bind_group_layout.clone(),
            ],
            push_constant_ranges: vec![],
            vertex: VertexState {
                shader: self.shader_handle.clone(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: self.shader_handle.clone(),
                entry_point: "fragment".into(),
                shader_defs: vec![],
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: key.primitive_topology(),
                front_face: bevy::render::render_resource::FrontFace::Ccw,
                cull_mode: None, // No culling for point cloud billboards.
                polygon_mode: bevy::render::render_resource::PolygonMode::Fill,
                ..default()
            },
            // depth_stencil: None,
            depth_stencil: Some(bevy::render::render_resource::DepthStencilState {
                format: bevy::core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: bevy::render::render_resource::CompareFunction::Greater,
                stencil: bevy::render::render_resource::StencilState::default(),
                bias: bevy::render::render_resource::DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            zero_initialize_workgroup_memory: false,
        })
    }
}

/// Prepared bind group data for point cloud material resources.
#[derive(Resource, Default)]
pub struct PreparedPointCloudBindGroups {
    pub material_bind_group: Option<BindGroup>,
}

/// Custom render command for point cloud rendering.
/// Combines Bevy's standard mesh binding commands with custom material binding.
type DrawPointCloud = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,       // Automatic view uniform binding.
    SetMeshBindGroup<1>,           // Automatic mesh uniform binding.
    SetPointCloudMaterialGroup<2>, // Custom material binding.
    DrawMesh,
);

/// Custom render command for binding point cloud material resources to @group(2).
/// Uses proper lifetime parameters and error string for current Bevy version.
pub struct SetPointCloudMaterialGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> bevy::render::render_phase::RenderCommand<P>
    for SetPointCloudMaterialGroup<I>
{
    type Param = SRes<PreparedPointCloudBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> bevy::render::render_phase::RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        if let Some(material_bind_group) = &bind_groups.material_bind_group {
            pass.set_bind_group(I, material_bind_group, &[]);
            bevy::render::render_phase::RenderCommandResult::Success
        } else {
            bevy::render::render_phase::RenderCommandResult::Failure("missing material bind group")
        }
    }
}

/// Phase item for sorted point cloud rendering.
/// Integrates with Bevy's render phase system for proper depth sorting.
pub struct PointCloudPhase {
    pub sort_key: FloatOrd,
    pub entity: (Entity, MainEntity),
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub batch_range: Range<u32>,
    pub extra_index: PhaseItemExtraIndex,
    pub indexed: bool,
}

impl PhaseItem for PointCloudPhase {
    fn entity(&self) -> Entity {
        self.entity.0
    }

    fn main_entity(&self) -> MainEntity {
        self.entity.1
    }

    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    fn batch_range(&self) -> &Range<u32> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        &mut self.batch_range
    }

    fn extra_index(&self) -> PhaseItemExtraIndex {
        self.extra_index.clone()
    }

    fn batch_range_and_extra_index_mut(&mut self) -> (&mut Range<u32>, &mut PhaseItemExtraIndex) {
        (&mut self.batch_range, &mut self.extra_index)
    }
}

impl SortedPhaseItem for PointCloudPhase {
    type SortKey = FloatOrd;

    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    fn sort(items: &mut [Self]) {
        items.sort_by_key(SortedPhaseItem::sort_key);
    }

    fn indexed(&self) -> bool {
        self.indexed
    }
}

impl CachedRenderPipelinePhaseItem for PointCloudPhase {
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

fn prepare_point_cloud_bind_groups(
    mut bind_groups: ResMut<PreparedPointCloudBindGroups>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    assets: Res<PointCloudAssets>,
    render_state: Res<PointCloudRenderState>,
    pipeline: Res<PointCloudPipeline>,
) {
    // Position texture
    let Some(position_gpu) = gpu_images.get(&assets.position_texture) else {
        warn!(
            "❌ Position texture not found in GPU images (handle: {:?})",
            assets.position_texture
        );
        return;
    };
    // Result texture
    let Some(final_gpu) = gpu_images.get(&assets.result_texture) else {
        warn!(
            "❌ Result texture not found in GPU images (handle: {:?})",
            assets.result_texture
        );
        return;
    };

    // Depth texture
    let Some(depth_gpu) = gpu_images.get(&assets.depth_texture) else {
        warn!(
            "❌ Depth texture not found in GPU images (handle: {:?})",
            assets.depth_texture
        );
        return;
    };

    // Create uniform buffer
    let material_uniform = create_point_cloud_material_uniform(&render_device, &render_state);

    // Build bind group
    let material_bind_group = render_device.create_bind_group(
        "point_cloud_material_bind_group",
        &pipeline.material_bind_group_layout,
        &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&position_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&position_gpu.sampler),
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&final_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&final_gpu.sampler),
            },
            BindGroupEntry {
                binding: 4,
                resource: BindingResource::TextureView(&depth_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 5,
                resource: BindingResource::Sampler(&depth_gpu.sampler),
            },
            BindGroupEntry {
                binding: 6,
                resource: material_uniform.as_entire_binding(),
            },
        ],
    );
    bind_groups.material_bind_group = Some(material_bind_group);
}

/// System to queue point cloud entities for rendering in the custom phase.
/// Identifies point cloud entities and creates phase items with proper sorting.
pub fn queue_point_cloud_meshes(
    point_cloud_draw_functions: Res<DrawFunctions<PointCloudPhase>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<PointCloudPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    point_cloud_pipeline: Res<PointCloudPipeline>,
    render_meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    mut point_cloud_phases: ResMut<ViewSortedRenderPhases<PointCloudPhase>>,
    mut views: Query<(&ExtractedView, &RenderVisibleEntities, &Msaa)>,
    point_cloud_entities: Query<(), With<PointCloudRenderable>>,
    mut debug_state: ResMut<PipelineDebugState>,
) {
    // Reset counters
    debug_state.entities_queued = 0;
    debug_state.mesh_instances_found = 0;
    debug_state.pipeline_specializations = 0;
    debug_state.phase_items_added = 0;
    debug_state.views_with_phases = 0;

    // Get total visible entities for progress tracking
    let total_visible = views
        .iter()
        .map(|(_, visible_entities, _)| visible_entities.iter::<Mesh3d>().count())
        .sum::<usize>();

    for (view, visible_entities, msaa) in &mut views {
        let Some(point_cloud_phase) = point_cloud_phases.get_mut(&view.retained_view_entity) else {
            continue;
        };

        debug_state.views_with_phases += 1;
        let draw_function = point_cloud_draw_functions.read().id::<DrawPointCloud>();
        let view_key = MeshPipelineKey::from_msaa_samples(msaa.samples())
            | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (render_entity, visible_entity) in visible_entities.iter::<Mesh3d>() {
            if point_cloud_entities.get(*render_entity).is_err() {
                continue;
            }

            debug_state.entities_queued += 1;

            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*visible_entity)
            else {
                continue;
            };

            debug_state.mesh_instances_found += 1;

            let Some(mesh) = render_meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let mut mesh_key = view_key;
            mesh_key |= MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());

            let pipeline_id = pipelines.specialize(
                &pipeline_cache,
                &point_cloud_pipeline,
                mesh_key,
                &mesh.layout,
            );

            let pipeline_id = match pipeline_id {
                Ok(id) => {
                    debug_state.pipeline_specializations += 1;
                    id
                }
                Err(err) => {
                    error!("Point cloud pipeline specialization failed: {}", err);
                    continue;
                }
            };

            let distance = rangefinder.distance_translation(&mesh_instance.translation);

            point_cloud_phase.add(PointCloudPhase {
                sort_key: FloatOrd(distance),
                entity: (*render_entity, *visible_entity),
                pipeline: pipeline_id,
                draw_function,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: mesh.indexed(),
            });

            debug_state.phase_items_added += 1;
        }
    }
}

/// View node for executing point cloud render phase.
/// Renders all queued point cloud entities using the render command system.
#[derive(Default)]
struct PointCloudRenderNode;

/// Modified render node with debug tracking integration.
/// Replace your existing PointCloudRenderNode implementation.
impl ViewNode for PointCloudRenderNode {
    type ViewQuery = (
        &'static ExtractedView,
        &'static ViewTarget,
        Option<&'static bevy::render::view::ViewDepthTexture>,
    );

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view, target, depth_stencil): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let Some(point_cloud_phases) =
            world.get_resource::<ViewSortedRenderPhases<PointCloudPhase>>()
        else {
            return Ok(());
        };

        let Some(point_cloud_phase) = point_cloud_phases.get(&view.retained_view_entity) else {
            return Ok(());
        };

        let view_entity = graph.view_entity();

        // Create depth attachment if depth texture exists
        let depth_stencil_attachment = depth_stencil.map(|depth_texture| {
            bevy::render::render_resource::RenderPassDepthStencilAttachment {
                view: &depth_texture.view(),
                depth_ops: Some(bevy::render::render_resource::Operations {
                    load: bevy::render::render_resource::LoadOp::Load, // Use existing depth from main pass
                    store: bevy::render::render_resource::StoreOp::Store,
                }),
                stencil_ops: None,
            }
        });

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("point_cloud_render_pass"),
            color_attachments: &[Some(target.get_color_attachment())],
            // depth_stencil_attachment: None,
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // Execute render phase with error conversion.
        let render_result = point_cloud_phase.render(&mut render_pass, world, view_entity);
        render_result.map_err(|draw_err| {
            error!("Point cloud phase render failed: {:?}", draw_err);
            NodeRunError::DrawError(draw_err)
        })
    }
}

/// Create bind group layout for point cloud material resources.
/// Defines texture and uniform buffer bindings for @group(2).
/// Called during pipeline initialization to cache layout for specialization.
fn create_point_cloud_material_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        "point_cloud_material_layout",
        &[
            // Position texture (binding 0).
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Position texture sampler (binding 1).
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(
                    bevy::render::render_resource::SamplerBindingType::NonFiltering,
                ),
                count: None,
            },
            // Final/result texture (binding 2).
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Final texture sampler (binding 3).
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(
                    bevy::render::render_resource::SamplerBindingType::NonFiltering,
                ),
                count: None,
            },
            // Depth texture (binding 4).
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: false },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Depth texture sampler (binding 5).
            BindGroupLayoutEntry {
                binding: 5,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(
                    bevy::render::render_resource::SamplerBindingType::NonFiltering,
                ),
                count: None,
            },
            // Point cloud material parameters (binding 6).
            BindGroupLayoutEntry {
                binding: 6,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    )
}

/// Create uniform buffer containing point cloud material parameters.
/// Matches shader PointCloudMaterial structure with params array layout.
fn create_point_cloud_material_uniform(
    render_device: &RenderDevice,
    render_state: &PointCloudRenderState,
) -> Buffer {
    use bytemuck::{Pod, Zeroable};

    // Match shader struct PointCloudMaterial with params: array<vec4<f32>, 3>.
    #[repr(C)]
    #[derive(Pod, Zeroable, Copy, Clone)]
    struct PointCloudMaterial {
        params: [[f32; 4]; 3], // params[0] = min_bounds+texture_size, params[1] = max_bounds, params[2] = camera_pos
    }

    let material = PointCloudMaterial {
        params: [
            // params[0] = min_bounds + texture_size
            [
                render_state.bounds_min.x,
                render_state.bounds_min.y,
                render_state.bounds_min.z,
                render_state.texture_size,
            ],
            // params[1] = max_bounds + padding
            [
                render_state.bounds_max.x,
                render_state.bounds_max.y,
                render_state.bounds_max.z,
                0.0,
            ],
            // params[2] = camera_position + padding
            [
                render_state.camera_position.x,
                render_state.camera_position.y,
                render_state.camera_position.z,
                0.0,
            ],
        ],
    };

    render_device.create_buffer_with_data(&bevy::render::render_resource::BufferInitDescriptor {
        label: Some("point_cloud_material_uniforms"),
        contents: bytemuck::cast_slice(&[material]),
        usage: BufferUsages::UNIFORM,
    })
}
