use crate::engine::assets::point_cloud_assets::PointCloudAssets;
use crate::engine::render::pipeline::point_cloud_render_pipeline::PointCloudRenderState;
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{SystemParamItem, lifetimeless::*},
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        Render, RenderApp, RenderSet,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{
            MeshVertexBufferLayoutRef, RenderMesh, RenderMeshBufferInfo, allocator::MeshAllocator,
        },
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        texture::GpuImage,
        view::ExtractedView,
    },
};
use bytemuck::{Pod, Zeroable};
const INSTANCED_ASSET_SHADER_PATH: &str = "shaders/instanced_assets.wgsl";

pub struct InstancedAssetRenderPlugin;

impl Plugin for InstancedAssetRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<InstancedAssetData>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_render_command::<Transparent3d, DrawInstancedAssets>()
            .init_resource::<SpecializedMeshPipelines<InstancedAssetPipeline>>()
            .init_resource::<PreparedInstancedAssetBindGroups>()
            .add_systems(
                Render,
                (
                    prepare_instanced_asset_bind_groups.in_set(RenderSet::PrepareBindGroups),
                    queue_instanced_assets.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app.init_resource::<InstancedAssetPipeline>();
    }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: [f32; 3],  // World position
    pub _padding1: f32,      // Alignment
    pub rotation: [f32; 4],  // Quaternion rotation
    pub uv_bounds: [f32; 4], // UV min/max for atlas sampling
    pub point_count: f32,    // Points in this asset's texture region
    pub _padding2: [f32; 3], // Alignment
}

#[derive(Component, Deref, Clone, ExtractComponent)]
pub struct InstancedAssetData(pub Vec<InstanceData>);

#[derive(Component)]
pub struct InstanceBuffer {
    pub buffer: Buffer,
    pub length: usize,
}

#[derive(Resource)]
struct InstancedAssetPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    material_bind_group_layout: BindGroupLayout,
}

impl FromWorld for InstancedAssetPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>();
        let render_device = world.resource::<RenderDevice>();

        Self {
            shader: world.load_asset(INSTANCED_ASSET_SHADER_PATH),
            mesh_pipeline: mesh_pipeline.clone(),
            material_bind_group_layout: create_instanced_asset_bind_group_layout(render_device),
        }
    }
}

impl SpecializedMeshPipeline for InstancedAssetPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![
                // Position (3 floats + 1 padding)
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                // Rotation quaternion
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 4,
                },
                // UV bounds
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
                // Point count + padding
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 6,
                },
            ],
        });

        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        // Update layout to include our material bind group
        descriptor
            .layout
            .push(self.material_bind_group_layout.clone());

        Ok(descriptor)
    }
}

#[derive(Resource, Default)]
struct PreparedInstancedAssetBindGroups {
    material_bind_group: Option<BindGroup>,
}

fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstancedAssetData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instanced_asset_data_buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

fn prepare_instanced_asset_bind_groups(
    mut bind_groups: ResMut<PreparedInstancedAssetBindGroups>,
    render_device: Res<RenderDevice>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    assets: Res<PointCloudAssets>,
    pipeline: Res<InstancedAssetPipeline>,
    render_state: Res<PointCloudRenderState>,
) {
    let Some(position_gpu) = gpu_images.get(&assets.asset_position_texture) else {
        return;
    };
    let Some(color_gpu) = gpu_images.get(&assets.asset_colour_class_texture) else {
        return;
    };

    let camera_pos = render_state.camera_position;
    let camera_uniform = render_device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("camera_uniform"),
        contents: bytemuck::cast_slice(&[camera_pos.x, camera_pos.y, camera_pos.z, 0.0f32]),
        usage: BufferUsages::UNIFORM,
    });

    let material_bind_group = render_device.create_bind_group(
        "instanced_asset_material_bind_group",
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
                resource: BindingResource::TextureView(&color_gpu.texture_view),
            },
            BindGroupEntry {
                binding: 3,
                resource: BindingResource::Sampler(&color_gpu.sampler),
            },
            BindGroupEntry {
                binding: 4,
                resource: camera_uniform.as_entire_binding(),
            },
        ],
    );

    bind_groups.material_bind_group = Some(material_bind_group);
}

fn queue_instanced_assets(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    instanced_asset_pipeline: Res<InstancedAssetPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<InstancedAssetPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstancedAssetData>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_instanced_assets = transparent_3d_draw_functions
        .read()
        .id::<DrawInstancedAssets>();

    for (view, msaa) in &views {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view.retained_view_entity)
        else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };

            let key =
                view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology());
            let pipeline = pipelines
                .specialize(
                    &pipeline_cache,
                    &instanced_asset_pipeline,
                    key,
                    &mesh.layout,
                )
                .unwrap();

            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_instanced_assets,
                distance: rangefinder.distance_translation(&mesh_instance.translation),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}

type DrawInstancedAssets = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetInstancedAssetMaterialGroup<2>,
    DrawMeshInstancedAssets,
);

struct SetInstancedAssetMaterialGroup<const I: usize>;

impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetInstancedAssetMaterialGroup<I> {
    type Param = SRes<PreparedInstancedAssetBindGroups>;
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        _entity: Option<()>,
        bind_groups: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_groups = bind_groups.into_inner();
        if let Some(material_bind_group) = &bind_groups.material_bind_group {
            pass.set_bind_group(I, material_bind_group, &[]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure("missing material bind group")
        }
    }
}

struct DrawMeshInstancedAssets;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstancedAssets {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes, render_mesh_instances, mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances.render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner().get(mesh_instance.mesh_asset_id) else {
            return RenderCommandResult::Skip;
        };
        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Skip;
        };
        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(index_buffer_slice.buffer.slice(..), 0, *index_format);
                pass.draw_indexed(
                    index_buffer_slice.range.start..(index_buffer_slice.range.start + count),
                    vertex_buffer_slice.range.start as i32,
                    0..instance_buffer.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(vertex_buffer_slice.range, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}

fn create_instanced_asset_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    render_device.create_bind_group_layout(
        "instanced_asset_material_layout",
        &[
            // Asset position texture
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Asset position sampler
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            // Asset color texture
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // Asset color sampler
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 4,
                visibility: ShaderStages::VERTEX,
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
