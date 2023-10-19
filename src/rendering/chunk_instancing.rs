Given the context, here is the code with comments added to describe the purpose of functions:

```rust
// Import necessary libraries and modules
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        primitives::Aabb,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        view::{ComputedVisibility, ExtractedView, Msaa},
        Extract, Render, RenderApp, RenderSet,
    },
};
use rand::Rng;

// Define the bundle struct for Chunk Instancing
#[derive(Bundle, Debug, Default)]
pub struct ChunkInstancingBundle {
    pub visibility: Visibility,
    pub computed: ComputedVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub mesh_handle: Handle<Mesh>,
    pub aabb: Aabb,
    pub chunk_instancing: ChunkInstancing,
    pub distance_culling: DistanceCulling,
    pub chunk: Chunk,
}

// Function to handle chunk distance culling based on camera position
fn chunk_distance_culling(
    mut query: Query<(&Transform, &mut Visibility, &DistanceCulling)>,
    query_camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_pos) = query_camera.get_single() {
        for (transform, mut visibility, distance_culling) in query.iter_mut() {
            if camera_pos.translation.distance(transform.translation) > distance_culling.distance {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Visible;
            }
        }
    }
}

// Struct to represent an instance in the Chunk Instancing system
#[derive(Clone, Debug)]
pub struct Instance {
    pub pos_xyz: [f32; 4],
}

// Struct to represent the Chunk Instancing component
#[derive(Component, Clone, Debug, Default)]
pub struct ChunkInstancing {
    pub instances: Vec<Instance>,
    pub base_color_texture: Handle<Image>,
    pub model_transform: Transform,
}

// Implementation of methods for Chunk Instancing
impl ChunkInstancing {
    // Create a new Chunk Instancing component with random instances
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

// Define the Chunk Instancing Plugin
pub struct ChunkInstancingPlugin;

// Implementation of Plugin trait for ChunkInstancingPlugin
impl Plugin for ChunkInstancingPlugin {
    // Build function to add systems and commands to the app
    fn build(&self, app: &mut App) {
        // Add systems for update and chunk distance culling
        app.add_systems(Update, chunk_distance_culling);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // Add render commands and systems for Chunk Instancing
        render_app
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(ExtractSchedule, extract_chunk_instancings)
            .add_systems(
                Render,
                prepare_chunk_instancing_instance_buffers
                    .in_set(RenderSet::Prepare),
            )
            .add_systems(Render, prepare_textures_bind_group.in_set(RenderSet::Prepare))
            .add_systems(Render, prepare_grass_chunk_bind_group.in_set(RenderSet::Prepare))
            .add_systems(Render, queue_custom.in_set(RenderSet::Queue));
    }

    // Finish function to initialize resources and finish setup
    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        // Initialize custom pipeline resource
        render_app.init_resource::<CustomPipeline>();
    }
}

// Function to extract Chunk Instancing entities and insert into world
fn extract_chunk_instancings(
    mut commands: Commands,
    mut previous_len: Local<usize>,
    mut query: Extract<Query<(Entity, &ComputedVisibility, &ChunkInstancing)>>,
) {
    if !query.is_empty() {
        let mut values = Vec::with_capacity(*previous_len);
        // Iterate over Chunk Instancing queries
        for (entity, computed_visibility, query_item) in query.iter_mut() {
            // Check if entity is visible
            if computed_visibility.is_visible() {
                // Create a tuple with entity and raw Chunk Instancing data
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
        // Insert or spawn Chunk Instancing entities in the world
        commands.insert_or_spawn_batch(values);
    }
}

// Structure to represent Chunk Instancing instances in raw format for GPU
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuInstance {
    pub pos_xyz: [f32; 4],
}

// Component to store Chunk Instancing instances in raw GPU format
#[derive(Component, Clone)]
pub struct GpuInstances(Vec<GpuInstance>);

// Component to store Chunk Instancing bind group data for GPU
#[derive(Component, Clone)]
struct GpuChunkBindGroupData {
    model_transform: [[f32; 4]; 4],
}

// Implementation of methods for Chunk Instancing
impl ChunkInstancing {
    // Convert Chunk Instancing instances to raw GPU format
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

    // Convert Chunk Instancing bind group data to raw GPU format
    fn to_raw_chunk_bind_group(&self) -> GpuChunkBindGroupData {
        GpuChunkBindGroupData {
            model_transform: self.model_transform.compute_matrix().to_cols_array_2d(),
        }
    }
}

// Chunk Instancing Plugin implementation
impl FromWorld for ChunkInstancingPlugin {
    // Implement FromWorld trait to define how ChunkInstancingPlugin is created from the World
    fn from_world(world: &mut World) -> Self {
        // Get necessary resources from the world
        let render_device = world.resource::<RenderDevice>();
        let chunk_instancing_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("grass_chunk_bind_group_layout"),
            });
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
```