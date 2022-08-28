use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem, SystemState},
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        primitives::Aabb,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ComputedVisibility, ExtractedView, Msaa},
        Extract, RenderApp, RenderStage,
    },
};

use rand::Rng;

use crate::camera::orbit::OrbitCamera;

use super::{Chunk, DistanceCulling};

//Bundle
#[derive(Bundle, Debug, Default)]
pub struct ChunkInstancingBundle {
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The computed visibility of the entity.
    pub computed: ComputedVisibility,
    /// The transform of the entity.
    pub transform: Transform,
    /// The global transform of the entity.
    pub global_transform: GlobalTransform,
    pub mesh_handle: Handle<Mesh>,
    pub aabb: Aabb,
    pub chunk_instancing: ChunkInstancing,
    pub distance_culling: DistanceCulling,
    pub chunk: Chunk,
}

fn chunk_distance_culling(
    mut query: Query<(&Transform, &mut Visibility, &DistanceCulling)>,
    query_camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_pos) = query_camera.get_single() {
        for (transform, mut visability, distance_culling) in query.iter_mut() {
            if camera_pos.translation.distance(transform.translation) > distance_culling.distance {
                visability.is_visible = false;
            } else {
                visability.is_visible = true;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Instance {
    pub pos_xyz: [f32; 4],
}

#[derive(Component, Clone, Debug, Default)]
pub struct ChunkInstancing {
    pub instances: Vec<Instance>, //[x,y,z, scale] Lower performance if using full Transforms
    pub base_color_texture: Handle<Image>,
    pub model_transform: Transform,
}

impl ChunkInstancing {
    pub fn new(
        nr_instances: u32,
        base_color_texture: Handle<Image>,
        model_transform: Transform,
        chunk_size: f32,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut instances = Vec::new();
        for _ in 0..nr_instances {
            let x = rng.gen::<f32>() * chunk_size;
            let y = rng.gen::<f32>() * chunk_size;
            let scale = rng.gen::<f32>() * 0.5 + 0.5;

            instances.push(Instance {
                pos_xyz: [x, y, 0.0, scale],
            });
        }

        Self {
            instances,
            base_color_texture,
            model_transform,
        }
    }
}

pub struct ChunkInstancingPlugin;

impl Plugin for ChunkInstancingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(chunk_distance_culling);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_system_to_stage(RenderStage::Extract, extract_chunk_instancings)
            .add_system_to_stage(
                RenderStage::Prepare,
                prepare_chunk_instancing_instance_buffers,
            )
            .add_system_to_stage(RenderStage::Prepare, prepare_textures_bind_group)
            .add_system_to_stage(RenderStage::Prepare, prepare_grass_chunk_bind_group)
            .add_system_to_stage(RenderStage::Queue, queue_custom);
    }
}

// ██████████████████████████████████████████████████████████████████████████████████████████████████████████████████
// █░░░░░░░░░░░░░░█░░░░░░░░██░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░░░███░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀░░██░░▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░░░░░█░░░░▄▀░░██░░▄▀░░░░█░░░░░░▄▀░░░░░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░░░█░░░░░░▄▀░░░░░░█
// █░░▄▀░░███████████░░▄▀▄▀░░▄▀▄▀░░███████░░▄▀░░█████░░▄▀░░████░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░█████████████░░▄▀░░█████
// █░░▄▀░░░░░░░░░░███░░░░▄▀▄▀▄▀░░░░███████░░▄▀░░█████░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░█████████████░░▄▀░░█████
// █░░▄▀▄▀▄▀▄▀▄▀░░█████░░▄▀▄▀▄▀░░█████████░░▄▀░░█████░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░█████████████░░▄▀░░█████
// █░░▄▀░░░░░░░░░░███░░░░▄▀▄▀▄▀░░░░███████░░▄▀░░█████░░▄▀░░░░░░▄▀░░░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░█████████████░░▄▀░░█████
// █░░▄▀░░███████████░░▄▀▄▀░░▄▀▄▀░░███████░░▄▀░░█████░░▄▀░░██░░▄▀░░█████░░▄▀░░██░░▄▀░░█░░▄▀░░█████████████░░▄▀░░█████
// █░░▄▀░░░░░░░░░░█░░░░▄▀░░██░░▄▀░░░░█████░░▄▀░░█████░░▄▀░░██░░▄▀░░░░░░█░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█████░░▄▀░░█████
// █░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀░░██░░▄▀▄▀░░█████░░▄▀░░█████░░▄▀░░██░░▄▀▄▀▄▀░░█░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█████░░▄▀░░█████
// █░░░░░░░░░░░░░░█░░░░░░░░██░░░░░░░░█████░░░░░░█████░░░░░░██░░░░░░░░░░█░░░░░░██░░░░░░█░░░░░░░░░░░░░░█████░░░░░░█████
// ██████████████████████████████████████████████████████████████████████████████████████████████████████████████████

//Make custom extract func in order to not clone instance data twice when using convinient abstract types for world side components
fn extract_chunk_instancings(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    mut query: Extract<Query<(Entity, &ComputedVisibility, &ChunkInstancing)>>,
) {
    if !query.is_empty() {
        let mut values = Vec::with_capacity(*previous_len);
        for (entity, computed_visibility, query_item) in query.iter_mut() {
            if computed_visibility.is_visible() {
                values.push((
                    entity,
                    (
                        query_item.to_raw_instances(),
                        query_item.to_raw_chunk_bind_group(),
                        query_item.base_color_texture.clone(),
                    ),
                ));
            }
        }
        *previous_len = values.len();
        commands.insert_or_spawn_batch(values);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuInstance {
    pub pos_xyz: [f32; 4],
}

#[derive(Component, Clone)]
pub struct GpuInstances(Vec<GpuInstance>);

#[derive(Component, Clone)]
struct GpuChunkBindGroupData {
    model_transform: [[f32; 4]; 4],
}

impl ChunkInstancing {
    fn to_raw_instances(&self) -> GpuInstances {
        GpuInstances(
            (&self.instances)
                .into_iter()
                .map(|v| GpuInstance {
                    pos_xyz: v.pos_xyz.clone(),
                })
                .collect(),
        )
    }
    fn to_raw_chunk_bind_group(&self) -> GpuChunkBindGroupData {
        GpuChunkBindGroupData {
            model_transform: self.model_transform.compute_matrix().to_cols_array_2d(),
        }
    }
}

// ██████████████████████████████████████████████████████████████████████████████████████████████████████████████████
// █░░░░░░░░░░░░░░█░░░░░░░░░░░░░░░░███░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░░░███░░░░░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░░░░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀░░████░░▄▀░░███░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░██░░▄▀░░█░░▄▀░░████░░▄▀░░███░░▄▀░░█████████
// █░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░░░░░█░░▄▀░░░░░░▄▀░░░░███░░▄▀░░░░░░░░░░█░░▄▀░░░░░░░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░▄▀░░░░███░░▄▀░░░░░░░░░░█
// █░░▄▀░░█████████░░▄▀░░██░░▄▀░░█████░░▄▀░░█████████░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░██░░▄▀░░█████░░▄▀░░█████████
// █░░▄▀░░█████████░░▄▀░░██░░▄▀░░░░░░█░░▄▀░░░░░░░░░░█░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░██░░▄▀░░░░░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░█████████░░▄▀░░██░░▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░██░░▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░░░░░█████████░░░░░░██░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░█████████░░░░░░██░░░░░░█░░░░░░██░░░░░░░░░░█░░░░░░░░░░░░░░█
// ██████████████████████████████████████████████████████████████████████████████████████████████████████████████████

#[derive(Component)]
pub struct ChunkInstancingInstanceBuffer {
    buffer: Buffer,
    length: usize,
}

fn prepare_chunk_instancing_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &GpuInstances)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, gpu_instances) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(gpu_instances.0.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands
            .entity(entity)
            .insert(ChunkInstancingInstanceBuffer {
                buffer,
                length: gpu_instances.0.len(),
            });
    }
}

#[derive(Component)]
pub struct ChunkInstancingBindGroup(BindGroup);

fn prepare_grass_chunk_bind_group(
    mut commands: Commands,
    query: Query<(Entity, &GpuChunkBindGroupData)>,
    render_device: Res<RenderDevice>,
    custom_pipeline: Res<CustomPipeline>,
) {
    for (entity, gpu_chunk) in &query {
        let chunk_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Chunk_instancing_buffer"),
            contents: bytemuck::cast_slice(&[gpu_chunk.model_transform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let chunk_instancing_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("Chunk_instancing_bindgroup"),
            layout: &custom_pipeline.chunk_instancing_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: chunk_buffer.as_entire_binding(),
            }],
        });
        commands
            .entity(entity)
            .insert(ChunkInstancingBindGroup(chunk_instancing_bind_group));
    }
}

#[derive(Component)]
pub struct TextureBindGroup(BindGroup);

pub fn prepare_textures_bind_group(
    render_device: Res<RenderDevice>,
    mut commands: Commands,
    custom_pipeline: Res<CustomPipeline>,
    image_query: Query<(Entity, &Handle<Image>), With<GpuInstances>>,
    gpu_images: Res<RenderAssets<Image>>,
) {
    for (e, texture_handle) in image_query.iter() {
        let gpu_image = gpu_images.get(&texture_handle).unwrap();

        let texture_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            layout: &custom_pipeline.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu_image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&gpu_image.sampler),
                },
            ],
            label: Some("growth_texture_bind_group"),
        });
        commands
            .entity(e)
            .insert(TextureBindGroup(texture_bind_group));
    }
}

// ██████████████████████████████████████████████████████████████████████████████
// █░░░░░░░░░░░░░░███░░░░░░██░░░░░░█░░░░░░░░░░░░░░█░░░░░░██░░░░░░█░░░░░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░██░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░█████████
// █░░▄▀░░██░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░██░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░██░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░██░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░█████████░░▄▀░░██░░▄▀░░█░░▄▀░░█████████
// █░░▄▀░░░░░░▄▀░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█
// ██████████████████████████████████████████████████████████████████████████████

#[allow(clippy::too_many_arguments)]
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<
        (Entity, &MeshUniform, &Handle<Mesh>, &Handle<Image>),
        With<GpuInstances>,
    >,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
    gpu_images: Res<RenderAssets<Image>>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in &mut views {
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle, image_handle) in &material_meshes {
            if let (Some(mesh), Some(_)) = (meshes.get(mesh_handle), gpu_images.get(&image_handle))
            {
                let key =
                    msaa_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let pipeline = pipelines
                    .specialize(&mut pipeline_cache, &custom_pipeline, key, &mesh.layout)
                    .unwrap();
                transparent_phase.add(Transparent3d {
                    entity,
                    pipeline,
                    draw_function: draw_custom,
                    distance: rangefinder.distance(&mesh_uniform.transform),
                });
            }
        }
    }
}

// █████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████
// █░░░░░░░░░░░░░░█░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░█████████░░░░░░░░░░█░░░░░░██████████░░░░░░█░░░░░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░█████████░░▄▀▄▀▄▀░░█░░▄▀░░░░░░░░░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░▄▀░░█░░░░▄▀░░░░█░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░█████████░░░░▄▀░░░░█░░▄▀▄▀▄▀▄▀▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░██░░▄▀░░███░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░█████████░░▄▀░░███████████░░▄▀░░███░░▄▀░░░░░░▄▀░░██░░▄▀░░█░░▄▀░░█████████
// █░░▄▀░░░░░░▄▀░░███░░▄▀░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░░░░░░░░░█░░▄▀░░███████████░░▄▀░░███░░▄▀░░██░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀▄▀▄▀▄▀▄▀░░███░░▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░███████████░░▄▀░░███░░▄▀░░██░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀░░░░░░░░░░███░░▄▀░░███░░▄▀░░░░░░░░░░█░░▄▀░░░░░░░░░░█░░▄▀░░███████████░░▄▀░░███░░▄▀░░██░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░███████████░░▄▀░░███░░▄▀░░█████████░░▄▀░░█████████░░▄▀░░███████████░░▄▀░░███░░▄▀░░██░░▄▀░░░░░░▄▀░░█░░▄▀░░█████████
// █░░▄▀░░█████████░░░░▄▀░░░░█░░▄▀░░█████████░░▄▀░░░░░░░░░░█░░▄▀░░░░░░░░░░█░░░░▄▀░░░░█░░▄▀░░██░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░░░░░░░░░█
// █░░▄▀░░█████████░░▄▀▄▀▄▀░░█░░▄▀░░█████████░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀▄▀▄▀░░█░░▄▀░░██░░░░░░░░░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀░░█
// █░░░░░░█████████░░░░░░░░░░█░░░░░░█████████░░░░░░░░░░░░░░█░░░░░░░░░░░░░░█░░░░░░░░░░█░░░░░░██████████░░░░░░█░░░░░░░░░░░░░░█
// █████████████████████████████████████████████████████████████████████████████████████████████████████████████████████████

pub struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    chunk_instancing_bind_group_layout: BindGroupLayout,
    texture_bind_group_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut system_state: SystemState<Res<RenderDevice>> = SystemState::new(world);
        let render_device = system_state.get_mut(world);

        //Instancing chunk
        let chunk_instancing_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false, //size will not change
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("grass_chunk_bind_group_layout"),
            });

        //Model texture
        let texture_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let asset_server = world.resource::<AssetServer>();
        asset_server.watch_for_changes().unwrap();
        let shader = asset_server.load("shaders/chunk_instancing.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            chunk_instancing_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}

impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        descriptor.primitive.cull_mode = None; //For grass
        descriptor.vertex.shader = self.shader.clone();
        descriptor.vertex.buffers.push(VertexBufferLayout {
            array_stride: std::mem::size_of::<GpuInstance>() as u64,
            step_mode: VertexStepMode::Instance,
            attributes: vec![VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 3, // shader locations 0-2 are taken up by Position, Normal and UV attributes
            }],
        });
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.chunk_instancing_bind_group_layout.clone(),
            self.texture_bind_group_layout.clone(),
        ]);

        Ok(descriptor)
    }
}

// █████████████████████████████████████████████████████████████████████████
// █░░░░░░░░░░░░███░░░░░░░░░░░░░░░░███░░░░░░░░░░░░░░█░░░░░░██████████░░░░░░█
// █░░▄▀▄▀▄▀▄▀░░░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░██████████░░▄▀░░█
// █░░▄▀░░░░▄▀▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░██████████░░▄▀░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀░░████░░▄▀░░███░░▄▀░░██░░▄▀░░█░░▄▀░░██████████░░▄▀░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░░░▄▀░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░██░░░░░░██░░▄▀░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀░░███░░▄▀▄▀▄▀▄▀▄▀░░█░░▄▀░░██░░▄▀░░██░░▄▀░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░▄▀░░░░███░░▄▀░░░░░░▄▀░░█░░▄▀░░██░░▄▀░░██░░▄▀░░█
// █░░▄▀░░██░░▄▀░░█░░▄▀░░██░░▄▀░░█████░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░▄▀░░░░░░▄▀░░█
// █░░▄▀░░░░▄▀▄▀░░█░░▄▀░░██░░▄▀░░░░░░█░░▄▀░░██░░▄▀░░█░░▄▀▄▀▄▀▄▀▄▀▄▀▄▀▄▀▄▀░░█
// █░░▄▀▄▀▄▀▄▀░░░░█░░▄▀░░██░░▄▀▄▀▄▀░░█░░▄▀░░██░░▄▀░░█░░▄▀░░░░░░▄▀░░░░░░▄▀░░█
// █░░░░░░░░░░░░███░░░░░░██░░░░░░░░░░█░░░░░░██░░░░░░█░░░░░░██░░░░░░██░░░░░░█
// █████████████████████████████████████████████████████████████████████████

type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    SetChunkInstancingBindGroup<2>,
    SetTextureBindGroup<3>,
    DrawMeshInstanced,
);

pub struct SetTextureBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetTextureBindGroup<I> {
    type Param = SQuery<Read<TextureBindGroup>>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        bind_group_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(bind_group) = bind_group_query.get_inner(item) {
            pass.set_bind_group(I, &bind_group.0, &[]);
            // return RenderCommandResult::Success;
        }

        RenderCommandResult::Success
    }
}

pub struct SetChunkInstancingBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetChunkInstancingBindGroup<I> {
    type Param = SQuery<Read<ChunkInstancingBindGroup>>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        bind_group_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Ok(bind_group) = bind_group_query.get_inner(item) {
            pass.set_bind_group(I, &bind_group.0, &[]);
            // return RenderCommandResult::Success;
        }

        RenderCommandResult::Success
    }
}

pub struct DrawMeshInstanced;

impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<ChunkInstancingInstanceBuffer>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, chunk_instancing_instance_buffer_query): SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();
        let instance_buffer = chunk_instancing_instance_buffer_query
            .get_inner(item)
            .unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(0..*count, 0, 0..instance_buffer.length as u32);
            }
            GpuBufferInfo::NonIndexed { vertex_count } => {
                pass.draw(0..*vertex_count, 0..instance_buffer.length as u32);
            }
        }
        RenderCommandResult::Success
    }
}
