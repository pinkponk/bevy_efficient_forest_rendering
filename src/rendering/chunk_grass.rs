use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::system::{lifetimeless::*, SystemParamItem, SystemState},
    math::prelude::*,
    pbr::{MeshPipeline, MeshPipelineKey, MeshUniform, SetMeshBindGroup, SetMeshViewBindGroup},
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        mesh::{GpuBufferInfo, MeshVertexBufferLayout},
        primitives::Aabb,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, EntityRenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        texture::ImageSampler,
        view::{ExtractedView, Msaa},
    },
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::Indices,
        render_resource::{PrimitiveTopology, ShaderType, SpecializedMeshPipelines},
        RenderApp, RenderStage,
    },
};
use bytemuck::{Pod, Zeroable};

use noise::{NoiseFn, Perlin, Seedable};

use super::{Chunk, DistanceCulling};

//Bundle
#[derive(Bundle, Debug, Default)]
pub struct ChunkGrassBundle {
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
    pub chunk_grass: ChunkGrass,
    pub distance_culling: DistanceCulling,
    pub chunk: Chunk,
}

fn update_time_for_custom_material(mut grass_chunks: Query<&mut ChunkGrass>, time: Res<Time>) {
    for mut grass_chunk in grass_chunks.iter_mut() {
        grass_chunk.time = time.seconds_since_startup() as f32;
    }
}

fn grass_chunk_distance_culling(
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

pub fn get_grass_straw_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(5);
    let mut normals = Vec::with_capacity(5);
    let mut uvs = Vec::with_capacity(5);

    positions.push([0., 0., 1.0]);
    positions.push([0.05, 0.0, 0.5]);
    positions.push([-0.05, 0.0, 0.5]);
    positions.push([0.05, 0.0, 0.0]);
    positions.push([-0.05, 0.0, 0.0]);

    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    normals.push([0.0, 1.0, 0.0]);

    uvs.push([0.5, 1.0]);
    uvs.push([1.0, 0.5]);
    uvs.push([0.0, 0.5]);
    uvs.push([1.0, 0.0]);
    uvs.push([0.0, 0.0]);

    let indices = vec![0, 1, 2, 1, 3, 2, 2, 3, 4];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

pub struct ChunkGrassPlugin;

impl Plugin for ChunkGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractComponentPlugin::<ChunkGrass>::extract_visible());
        app.add_plugin(ExtractResourcePlugin::<GrowthTextures>::default());
        app.add_plugin(ExtractResourcePlugin::<GridConfig>::default());
        app.insert_resource(GridConfig::default());
        app.insert_resource(GrowthTextures::default());
        app.add_system(update_time_for_custom_material);
        app.add_system(grass_chunk_distance_culling);

        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<CustomPipeline>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<GridConfigBindGroup>()
            .init_resource::<GrowthTexturesBindGroup>()
            .add_system_to_stage(RenderStage::Queue, queue_custom_pipeline)
            .add_system_to_stage(RenderStage::Prepare, prepare_grid_config_bind_group)
            .add_system_to_stage(RenderStage::Prepare, prepare_grass_chunk_bind_group)
            .add_system_to_stage(RenderStage::Prepare, prepare_growth_textures_bind_group);
    }
}

#[derive(Clone, Component, Default)]
pub struct GrowthTextures {
    pub growth_texture_array_handle: Option<Handle<Image>>,
}

impl GrowthTextures {
    pub fn new(images: &mut ResMut<Assets<Image>>) -> Self {
        let size = 100;
        let scale = 255.0;
        let mut data = Vec::new();
        let pattern_scale = 0.1;
        let nr_textures = 2;
        for i in 0..nr_textures {
            let perlin = Perlin::new().set_seed(i + 1); // from -1 to 1

            for y in 0..size {
                for x in 0..size {
                    let noise =
                        perlin.get([x as f64 * pattern_scale, y as f64 * pattern_scale]) as f32;
                    let value = (noise + 1.0) / 2.0 * scale;
                    // value = (value - 100.0).max(0.0); //Truncate short grass to 0.0
                    data.push(value as u8)
                }
            }
        }

        let image = Image::new(
            Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: nr_textures,
            },
            TextureDimension::D2,
            data,
            TextureFormat::R8Unorm,
        );

        Self {
            growth_texture_array_handle: Some(images.add(image)),
        }
    }
}

#[derive(Clone, Default)]
pub struct GridConfig {
    pub grid_center_xy: [f32; 2], //Assume axis aligned grid otherwise need to calc homogenous coordinate matrix
    pub grid_half_extents: [f32; 2],
}

impl GridConfig {
    pub fn get_size(self: &Self) -> Vec2 {
        Vec2::new(
            self.grid_half_extents[0] * 2.0,
            self.grid_half_extents[1] * 2.0,
        )
    }
}

#[derive(TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-42ab-8225-97e2a3f056e0"] //Dont know why this is needed?
pub struct ChunkGrass {
    pub time: f32,
    pub healthy_tip_color: Color,
    pub healthy_middle_color: Color,
    pub healthy_base_color: Color,
    pub unhealthy_tip_color: Color,
    pub unhealthy_middle_color: Color,
    pub unhealthy_base_color: Color,
    pub chunk_xy: [f32; 2],
    pub chunk_half_extents: [f32; 2],
    pub nr_instances: u32,
    pub growth_texture_id: i32,
    pub height_modifier: f32,
    pub scale: f32,
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

impl ExtractComponent for ChunkGrass {
    type Query = &'static ChunkGrass;
    type Filter = ();

    fn extract_component(item: bevy::ecs::query::QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

impl ExtractResource for GrowthTextures {
    type Source = GrowthTextures;

    fn extract_resource(res: &Self::Source) -> Self {
        res.clone()
    }
}

impl ExtractResource for GridConfig {
    type Source = GridConfig;

    fn extract_resource(res: &Self::Source) -> Self {
        res.clone()
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
pub struct ChunkGrassBindGroup {
    pub grass_chunk_bind_group: BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
pub struct GpuChunkGrass {
    //Every struct element needs to be divisable with 16 bytes or padding needs to be added. This could probably be done some other way...
    //https://www.w3.org/TR/WGSL/#alignment-and-size
    pub time: [f32; 4],
    // pub wind_dir: [f32; 2], //Not used yet
    // pub wind_power: f32, //Not used yet
    pub healthy_tip_color: [f32; 4],
    pub healthy_middle_color: [f32; 4],
    pub healthy_base_color: [f32; 4],
    pub unhealthy_tip_color: [f32; 4],
    pub unhealthy_middle_color: [f32; 4],
    pub unhealthy_base_color: [f32; 4],

    pub chunk_xy: [f32; 2],
    pub chunk_half_extents: [f32; 2],
    pub growth_texture_id: [i32; 4],
    pub height_modifier: [f32; 4],
    pub scale: [f32; 4],
}

impl ChunkGrass {
    fn to_raw(self: &Self) -> GpuChunkGrass {
        GpuChunkGrass {
            time: [self.time, 0.0, 0.0, 0.0],
            // wind_dir: [0.5, -0.5],
            // wind_power: 1.0,
            healthy_tip_color: self.healthy_tip_color.as_linear_rgba_f32().into(),
            healthy_middle_color: self.healthy_middle_color.as_linear_rgba_f32().into(),
            healthy_base_color: self.healthy_base_color.as_linear_rgba_f32().into(),
            unhealthy_tip_color: self.unhealthy_tip_color.as_linear_rgba_f32().into(),
            unhealthy_middle_color: self.unhealthy_middle_color.as_linear_rgba_f32().into(),
            unhealthy_base_color: self.unhealthy_base_color.as_linear_rgba_f32().into(),

            chunk_xy: self.chunk_xy,
            chunk_half_extents: self.chunk_half_extents,
            growth_texture_id: [self.growth_texture_id, 0, 0, 0], //To lazy to understand alingment XD
            height_modifier: [self.height_modifier, 0.0, 0.0, 0.0],
            scale: [self.scale, 0.0, 0.0, 0.0],
        }
    }
}

fn prepare_grass_chunk_bind_group(
    mut commands: Commands,
    query: Query<(Entity, &ChunkGrass)>,
    render_device: Res<RenderDevice>,
    custom_pipeline: Res<CustomPipeline>,
) {
    for (entity, grass_chunk) in &query {
        let grass_chunk_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("Grass_chunk_buffer"),
            contents: bytemuck::cast_slice(&[grass_chunk.to_raw()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let grass_chunk_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("Grass_chunk_bindgroup"),
            layout: &custom_pipeline.grass_chunk_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: grass_chunk_buffer.as_entire_binding(),
            }],
        });
        commands.entity(entity).insert(ChunkGrassBindGroup {
            grass_chunk_bind_group,
        });
    }
}

#[derive(Default)]
pub struct GrowthTexturesBindGroup {
    pub bind_group: Option<BindGroup>,
}

pub fn prepare_growth_textures_bind_group(
    render_device: Res<RenderDevice>,
    custom_pipeline: Res<CustomPipeline>,
    mut growth_textures_bind_group: ResMut<GrowthTexturesBindGroup>,
    growth_textures: Res<GrowthTextures>,
    images: Res<RenderAssets<Image>>,
) {
    if let Some(image_handle) = growth_textures.growth_texture_array_handle.as_ref() {
        if let Some(image) = images.get(&image_handle.clone_weak()) {
            let growth_texture_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                layout: &custom_pipeline.growth_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(
                            &render_device.create_sampler(&ImageSampler::linear_descriptor()),
                        ),
                    },
                ],
                label: Some("growth_texture_bind_group"),
            });

            growth_textures_bind_group.as_mut().bind_group = Some(growth_texture_bind_group);
        }
    }
}

#[derive(Default)]
pub struct GridConfigBindGroup {
    pub grid_config_bind_group: Option<BindGroup>,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
struct GpuGridConfig {
    pub grid_center_xy: [f32; 2], //Assume axis aligned grid otherwise need to calc homogenous coordinate matrix
    pub grid_half_extents: [f32; 2],
}

impl GridConfig {
    fn to_raw(self: &Self) -> GpuGridConfig {
        GpuGridConfig {
            grid_center_xy: self.grid_center_xy.clone(),
            grid_half_extents: self.grid_half_extents.clone(),
        }
    }
}

fn prepare_grid_config_bind_group(
    render_device: Res<RenderDevice>,
    mut grid_config_bind_group_res: ResMut<GridConfigBindGroup>,
    grid_config: Res<GridConfig>,
    custom_pipeline: Res<CustomPipeline>,
) {
    if grid_config.is_changed() {
        let grid_config_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("grid_config_buffer"),
            contents: bytemuck::cast_slice(&[grid_config.to_raw()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let grid_config_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("grid_config_bindgroup"),
            layout: &custom_pipeline.grid_config_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: grid_config_buffer.as_entire_binding(),
            }],
        });

        grid_config_bind_group_res.as_mut().grid_config_bind_group = Some(grid_config_bind_group);
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
fn queue_custom_pipeline(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    mut pipeline_cache: ResMut<PipelineCache>,
    meshes: Res<RenderAssets<Mesh>>,
    material_meshes: Query<(Entity, &MeshUniform, &Handle<Mesh>), With<ChunkGrass>>,
    mut views: Query<(&ExtractedView, &mut RenderPhase<Transparent3d>)>,
    growth_textures: Res<GrowthTextures>,
) {
    let draw_custom = transparent_3d_draw_functions
        .read()
        .get_id::<DrawCustom>()
        .unwrap();

    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples);

    for (view, mut transparent_phase) in &mut views {
        let rangefinder = view.rangefinder3d();
        for (entity, mesh_uniform, mesh_handle) in &material_meshes {
            if let Some(mesh) = meshes.get(mesh_handle) {
                //Only render stuff if there is a texture handle
                if growth_textures.growth_texture_array_handle.is_some() {
                    let key = msaa_key
                        | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
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
    grass_chunk_bind_group_layout: BindGroupLayout,
    growth_bind_group_layout: BindGroupLayout,
    grid_config_bind_group_layout: BindGroupLayout,
}

impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let mut system_state: SystemState<Res<RenderDevice>> = SystemState::new(world);
        let render_device = system_state.get_mut(world);

        //NEW grass STUFF
        let grass_chunk_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false, //size will not change
                        // min_binding_size: Some(GpuGrassMaterial::min_size()),
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("grass_chunk_bind_group_layout"),
            });
        //Grass END

        //NEW Texture STUFF
        let growth_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2Array,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        //TEXTURE END

        //NEW grid config STUFF
        let grid_config_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false, //size will not change
                        // min_binding_size: Some(GpuGrassMaterial::min_size()),
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("grid_config_bind_group_layout"),
            });
        //grid END

        let asset_server = world.resource::<AssetServer>();
        asset_server.watch_for_changes().unwrap();
        let shader = asset_server.load("shaders/grass.wgsl");

        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
            grass_chunk_bind_group_layout,
            growth_bind_group_layout,
            grid_config_bind_group_layout,
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
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor.layout = Some(vec![
            self.mesh_pipeline.view_layout.clone(),
            self.mesh_pipeline.mesh_layout.clone(),
            self.grass_chunk_bind_group_layout.clone(),
            self.growth_bind_group_layout.clone(),
            self.grid_config_bind_group_layout.clone(),
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
    SetChunkGrassBindGroup<2>,
    SetGrowthTexturesBindGroup<3>,
    SetGridConfigBindGroup<4>,
    DrawMeshInstanced,
);

pub struct SetChunkGrassBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetChunkGrassBindGroup<I> {
    type Param = SQuery<Read<ChunkGrassBindGroup>>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        grass_bind_group_query: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let grass_chunk_bind_group = grass_bind_group_query.get_inner(item).unwrap();
        pass.set_bind_group(I, &grass_chunk_bind_group.grass_chunk_bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub struct SetGrowthTexturesBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetGrowthTexturesBindGroup<I> {
    type Param = SRes<GrowthTexturesBindGroup>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        bind_group_res: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(bindgroup) = bind_group_res.into_inner().bind_group.as_ref() {
            pass.set_bind_group(I, bindgroup, &[]);
            return RenderCommandResult::Success;
        }
        RenderCommandResult::Failure
    }
}

pub struct SetGridConfigBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetGridConfigBindGroup<I> {
    type Param = SRes<GridConfigBindGroup>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        bind_group_res: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(
            I,
            &bind_group_res
                .into_inner()
                .grid_config_bind_group
                .as_ref()
                .unwrap(),
            &[],
        );
        RenderCommandResult::Success
    }
}

pub struct DrawMeshInstanced;

impl EntityRenderCommand for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<Mesh>>,
        SQuery<Read<Handle<Mesh>>>,
        SQuery<Read<ChunkGrass>>,
    );
    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        (meshes, mesh_query, grass_chunk): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let mesh_handle = mesh_query.get(item).unwrap();

        let gpu_mesh = match meshes.into_inner().get(mesh_handle) {
            Some(gpu_mesh) => gpu_mesh,
            None => return RenderCommandResult::Failure,
        };

        pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));

        match &gpu_mesh.buffer_info {
            GpuBufferInfo::Indexed {
                buffer,
                index_format,
                count,
            } => {
                pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                pass.draw_indexed(
                    0..*count,
                    0,
                    0..grass_chunk.get(item).unwrap().nr_instances as u32,
                );
            }
            _ => {
                panic!("Non indexed not supported")
            }
        }
        RenderCommandResult::Success
    }
}
