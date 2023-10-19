```rust
use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
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
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        texture::ImageSampler,
        view::{ExtractedView, Msaa},
        Render,
    },
    render::{
        extract_component::ExtractComponentPlugin,
        mesh::Indices,
        render_resource::{PrimitiveTopology, ShaderType, SpecializedMeshPipelines},
        RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

use noise::{NoiseFn, Perlin};

use super::{Chunk, DistanceCulling};

/// Bundle for ChunkGrass components
#[derive(Bundle, Debug, Default)]
pub struct ChunkGrassBundle {
    pub visibility: Visibility,
    pub computed: ComputedVisibility,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub mesh_handle: Handle<Mesh>,
    pub aabb: Aabb,
    pub chunk_grass: ChunkGrass,
    pub distance_culling: DistanceCulling,
    pub chunk: Chunk,
}

/// Update the elapsed time for the custom material.
fn update_time_for_custom_material(mut grass_chunks: Query<&mut ChunkGrass>, time: Res<Time>) {
    for mut grass_chunk in grass_chunks.iter_mut() {
        grass_chunk.time = time.elapsed_seconds();
    }
}

/// Apply distance culling to the grass chunks.
fn grass_chunk_distance_culling(
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

/// Get the mesh for the grass straw.
fn get_grass_straw_mesh() -> Mesh {
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

/// Plugin for ChunkGrass
pub struct ChunkGrassPlugin;

impl Plugin for ChunkGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ChunkGrass>::extract_visible());
        app.add_plugins(ExtractResourcePlugin::<GrowthTextures>::default());
        app.add_plugins(ExtractResourcePlugin::<GridConfig>::default());
        app.insert_resource(GridConfig::default());
        app.insert_resource(GrowthTextures::default());
        app.add_systems(Update, update_time_for_custom_material);
        app.add_systems(Update, grass_chunk_distance_culling);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<GridConfigBindGroup>()
            .init_resource::<GrowthTexturesBindGroup>()
            .add_systems(Render, queue_custom_pipeline.in_set(RenderSet::Queue))
            .add_systems(
                Render,
                prepare_grid_config_bind_group.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                prepare_grass_chunk_bind_group.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                prepare_growth_textures_bind_group.in_set(RenderSet::Prepare),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app.init_resource::<CustomPipeline>();
    }
}

/// Resource containing the growth textures
#[derive(Clone, Default, Resource)]
pub struct GrowthTextures {
    pub growth_texture_array_handle: Option<Handle<Image>>,
}

impl GrowthTextures {
    /// Create a new GrowthTextures resource
    fn new(images: &mut ResMut<Assets<Image>>) -> Self {
        let size = 100;
        let scale = 255.0;
        let mut data = Vec::new();
        let pattern_scale = 0.05;
        let nr_textures = 2;
        for i in 0..nr_textures {
            let perlin = Perlin::new(i + 1);

            for y in 0..size {
                for x in 0..size {
                    let noise =
                        perlin.get([x as f64 * pattern_scale, y as f64 * pattern_scale]) as f32;
                    let value = (noise + 1.0) / 2.0 * scale;
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

/// Resource containing the grid configuration
#[derive(Clone, Default, Resource)]
pub struct GridConfig {
    pub grid_center_xy: [f32; 2],
    pub grid_half_extents: [f32; 2],
}

impl GridConfig {
    /// Get the size of the grid
    fn get_size(self: &Self) -> Vec2 {
        Vec2::new(
            self.grid_half_extents[0] * 2.0,
            self.grid_half_extents[1] * 2.0,
        )
    }
}

/// Component representing the grass chunk
#[derive(TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-42ab-8225-97e2a3f056e0"]
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

/// Apply time updates to the custom material.
fn update_time_for_custom_material(mut grass_chunks: Query<&mut ChunkGrass>, time: Res<Time>) {
    for mut grass_chunk in grass_chunks.iter_mut() {
        grass_chunk.time = time.elapsed_seconds();
    }
}

/// Apply distance culling to the grass chunks.
fn grass_chunk_distance_culling(
    mut query: Query<(&Transform, &mut Visibility, &DistanceCulling)>,
    query_camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_pos) = query_camera.get_single() {
        for (transform, mut visability, distance_culling) in query.iter_mut() {
            if camera_pos.translation.distance(transform.translation) > distance_culling.distance {
                *visability = Visibility::Hidden;
            } else {
                *visability = Visibility::Visible;
            }
        }
    }
}

/// Get the mesh for the grass straw.
fn get_grass_straw_mesh() -> Mesh {
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

/// Plugin for ChunkGrass
pub struct ChunkGrassPlugin;

impl Plugin for ChunkGrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<ChunkGrass>::extract_visible());
        app.add_plugins(ExtractResourcePlugin::<GrowthTextures>::default());
        app.add_plugins(ExtractResourcePlugin::<GridConfig>::default());
        app.insert_resource(GridConfig::default());
        app.insert_resource(GrowthTextures::default());
        app.add_systems(Update, update_time_for_custom_material);
        app.add_systems(Update, grass_chunk_distance_culling);

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .init_resource::<GridConfigBindGroup>()
            .init_resource::<GrowthTexturesBindGroup>()
            .add_systems(Render, queue_custom_pipeline.in_set(RenderSet::Queue))
            .add_systems(
                Render,
                prepare_grid_config_bind_group.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                prepare_grass_chunk_bind_group.in_set(RenderSet::Prepare),
            )
            .add_systems(
                Render,
                prepare_growth_textures_bind_group.in_set(RenderSet::Prepare),
            );
    }

    fn finish(&self, app: &mut App) {
        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app.init_resource::<CustomPipeline>();
    }
}

/// Resource containing the growth textures
#[derive(Clone, Default, Resource)]
pub struct GrowthTextures {
    pub growth_texture_array_handle: Option<Handle<Image>>,
}

impl GrowthTextures {
    /// Create a new GrowthTextures resource
    fn new(images: &mut ResMut<Assets<Image>>) -> Self {
        let size = 100;
        let scale = 255.0;
        let mut data = Vec::new();
        let pattern_scale = 0.05;
        let nr_textures = 2;
        for i in 0..nr_textures {
            let perlin = Perlin::new(i + 1);

            for y in 0..size {
                for x in 0..size {
                    let noise =
                        perlin.get([x as f64 * pattern_scale, y as f64 * pattern_scale]) as f32;
                    let value = (noise + 1.0) / 2.0 * scale;
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

/// Resource containing the grid configuration
#[derive(Clone, Default, Resource)]
pub struct GridConfig {
    pub grid_center_xy: [f32; 2],
    pub grid_half_extents: [f32; 2],
}

impl GridConfig {
    /// Get the size of the grid
    fn get_size(self: &Self) -> Vec2 {
        Vec2::new(
            self.grid_half_extents[0] * 2.0,
            self.grid_half_extents[1] * 2.0,
        )
    }
}

/// Component representing the grass chunk
#[derive(TypeUuid, Debug, Clone, Component, Default)]
#[uuid = "f690fdae-d598-42ab-8225-97e2a3f056e0"]
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

/// Struct representing the bind group for ChunkGrass
pub struct ChunkGrassBindGroup {
    pub grass_chunk_bind_group: BindGroup,
}

/// The GpuChunkGrass struct represents the chunk grass data to be sent to the GPU
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, ShaderType)]
pub struct GpuChunkGrass {
    pub time: [f32; 4],
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
    /// Convert the ChunkGrass struct to the GpuChunkGrass struct
    fn to_raw(self: &Self) -> GpuChunkGrass {
        GpuChunkGrass {
            time: [self.time, 0.0, 0.0, 0.0],
            healthy_tip_color: self.healthy_tip_color.as_linear_rgba_f32().into(),
            healthy_middle_color: self.healthy_middle_color.as_linear_rgba_f32().into(),
            healthy_base_color: self.healthy_base_color.as_linear_rgba_f32().into(),
            unhealthy_tip_color: self.unhealthy_tip_color.as_linear_rgba_f32().into(),
            unhealthy_middle_color: self.unhealthy_middle_color.as_linear_rgba_f32().into(),
            unhealthy_base_color: self.unhealthy_base_color.as_linear_rgba_f32().into(),
            chunk_xy: self.chunk_xy,
            chunk_half_extents: self.chunk_half_extents,
            growth_texture_id: [self.growth_texture_id, 0, 0, 0],
            height_modifier: [self.height_modifier, 0.0, 0.0, 0.0],
            scale: [self.scale, 0.0, 0.0, 0.0],
        }
    }
}

/// Prepare the grass