use std::time::Duration;

use bevy::{
    asset::ChangeWatcher,
    gltf::{Gltf, GltfMesh},
    math::prelude::*,
    math::Vec3A,
    prelude::*,
    render::{
        mesh::VertexAttributeValues,
        primitives::Aabb,
        render_resource::{AddressMode, FilterMode, SamplerDescriptor},
        texture::ImageSampler,
    },
    window::WindowResolution,
};

use itertools::Itertools;

use bevy_efficient_forest_rendering::rendering::{
    chunk_grass::{
        get_grass_straw_mesh, ChunkGrass, ChunkGrassBundle, ChunkGrassPlugin, GridConfig,
        GrowthTextures,
    },
    chunk_instancing::{ChunkInstancing, ChunkInstancingBundle, ChunkInstancingPlugin},
    Chunk, DistanceCulling,
};

use bevy_asset_loader::prelude::*;

mod helpers;

use helpers::*;

#[cfg(target_family = "wasm")]
use bevy_web_fullscreen::FullViewportPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    AssetLoading,
    InGame,
}

#[derive(AssetCollection, Resource)]
pub struct MyGltfAssets {
    #[asset(path = "mushroom.glb")]
    mushroom: Handle<Gltf>,
    #[asset(path = "tree.glb")]
    tree: Handle<Gltf>,
    #[asset(path = "bush.glb")]
    bush: Handle<Gltf>,
    #[asset(path = "rock.glb")]
    rock: Handle<Gltf>,
}

#[derive(AssetCollection, Resource)]
pub struct MyImageAssets {
    // #[asset(path = "grass_ground_texture.png")]
    #[asset(path = "Ground066_1K-PNG_Color.png")]
    grass_color: Handle<Image>,
    #[asset(path = "Ground066_1K-PNG_NormalGL.png")]
    grass_normal: Handle<Image>,
    #[asset(path = "Ground066_1K-PNG_AmbientOcclusion.png")]
    grass_occlusion: Handle<Image>,
    #[asset(path = "Ground066_1K-PNG_Roughness.png")]
    grass_roughness: Handle<Image>,
    #[asset(path = "Ground066_1K-PNG_Displacement.png")]
    grass_displacement: Handle<Image>,
}

const NR_SIDE_CHUNKS: u32 = 20;
const INSTANCE_DENSITY: i32 = 1; //4
const CHUNK_SIZE: f32 = 30.;

fn main() {
    let mut app = App::new();

    #[cfg(target_family = "wasm")]
    app.add_plugin(FullViewportPlugin);

    app.add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::InGame),
        )
        .add_collection_to_loading_state::<_, MyGltfAssets>(GameState::AssetLoading)
        .add_collection_to_loading_state::<_, MyImageAssets>(GameState::AssetLoading)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(Duration::from_millis(200)),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::At(IVec2::new(1450, 550)),
                        resolution: WindowResolution::new(1000.0, 1000.0),
                        present_mode: bevy::window::PresentMode::AutoNoVsync, //Dont cap at 60 fps
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::rgb(0.7, 0.8, 0.8)))
        .add_plugins(ChunkInstancingPlugin)
        .add_plugins(ChunkGrassPlugin)
        .add_plugins(HelpersPlugin)
        .add_systems(OnEnter(GameState::InGame), setup_ground_grass)
        .add_systems(OnEnter(GameState::InGame), setup_plants)
        .run();
}

fn setup_ground_grass(
    mut commands: Commands,
    mut grid_config: ResMut<GridConfig>,
    mut growth_texture: ResMut<GrowthTextures>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    my_image_assets: Res<MyImageAssets>,
) {
    //light
    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 3.05,
    // });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 30000.0,
            shadows_enabled: false, //Weird things happen
            ..default()
        },
        ..default()
    });

    //Set map size
    *grid_config = GridConfig {
        grid_center_xy: [0.0, 0.0],
        grid_half_extents: [
            NR_SIDE_CHUNKS as f32 * CHUNK_SIZE / 2.0,
            NR_SIDE_CHUNKS as f32 * CHUNK_SIZE / 2.0,
        ],
    };

    //Growth Textures
    *growth_texture = GrowthTextures::new(&mut images);

    //Ground grass texture
    for image_handle_ in [
        my_image_assets.grass_color.clone(),
        my_image_assets.grass_normal.clone(),
        my_image_assets.grass_occlusion.clone(),
        my_image_assets.grass_roughness.clone(),
        my_image_assets.grass_displacement.clone(),
    ]
    .iter()
    {
        images.get_mut(image_handle_).unwrap().sampler_descriptor =
            ImageSampler::Descriptor(SamplerDescriptor {
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Linear,
                ..default()
            });
    }

    let mut ground_mesh = Mesh::from(shape::Quad {
        size: grid_config.get_size(),
        flip: false,
    });
    if let Some(VertexAttributeValues::Float32x2(uvs)) =
        ground_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        for uv in uvs {
            uv[0] *= grid_config.grid_half_extents[0] / 4.0; //How dense texture should be sampled
            uv[1] *= grid_config.grid_half_extents[1] / 4.0;
        }
    }

    // Ground
    commands
        .spawn(PbrBundle {
            transform: Transform {
                translation: (Vec3::new(0., 0., 0.)),
                rotation: Quat::from_rotation_x(0.0_f32.to_radians()),
                ..Default::default()
            },
            mesh: meshes.add(Mesh::from(ground_mesh)),
            material: materials.add(StandardMaterial {
                // base_color: Color::rgb(0.34, 0.53, 0.255), //Adjust ground color
                // base_color: Color::WHITE,
                base_color: Color::rgb(0.6, 0.75, 0.5), //Adjust ground color
                base_color_texture: Some(my_image_assets.grass_color.clone()),
                depth_map: Some(my_image_assets.grass_displacement.clone()),
                normal_map_texture: Some(my_image_assets.grass_normal.clone()),
                occlusion_texture: Some(my_image_assets.grass_occlusion.clone()),
                metallic_roughness_texture: Some(my_image_assets.grass_roughness.clone()),
                cull_mode: None,
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                metallic: 0.0,
                ..default()
            }),
            ..default()
        })
        .insert(Name::new("Ground"));

    let nr_instances = (CHUNK_SIZE * CHUNK_SIZE * INSTANCE_DENSITY as f32) as u32;
    let mut tot_instances_grass = 0;
    for (chunk_x, chunk_y) in (0..NR_SIDE_CHUNKS).cartesian_product(0..NR_SIDE_CHUNKS) {
        let chunk_x_pos = chunk_x as f32 * CHUNK_SIZE - CHUNK_SIZE * NR_SIDE_CHUNKS as f32 / 2.0;
        let chunk_y_pos = chunk_y as f32 * CHUNK_SIZE - CHUNK_SIZE * NR_SIDE_CHUNKS as f32 / 2.0;
        let chunk = Chunk {
            chunk_xy: [chunk_x, chunk_y],
        };

        commands.spawn(ChunkGrassBundle {
            transform: Transform::from_xyz(chunk_x_pos, chunk_y_pos, 0.0),
            mesh_handle: meshes.add(get_grass_straw_mesh()),
            aabb: Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(CHUNK_SIZE, CHUNK_SIZE, 0.0), //Why do I need full chunk_size here?!
            },
            chunk_grass: ChunkGrass {
                time: 0.0,
                // healthy_tip_color: *[Color::ANTIQUE_WHITE, Color::RED].choose(&mut rand::thread_rng()).unwrap(),
                healthy_tip_color: Color::rgb(0.66, 0.79 + 0.2, 0.34), //Color::rgb(0.95, 0.91, 0.81),
                healthy_middle_color: Color::rgb(0.40, 0.60, 0.3),
                healthy_base_color: Color::rgb(0.22, 0.40, 0.255),

                unhealthy_tip_color: Color::rgb(0.9, 0.95, 0.14), //Should add favorability map
                unhealthy_middle_color: Color::rgb(0.52, 0.57, 0.25),
                unhealthy_base_color: Color::rgb(0.22, 0.40, 0.255), //Color::rgb(0.22, 0.40, 0.255),

                chunk_xy: [chunk_x_pos, chunk_y_pos],
                chunk_half_extents: [CHUNK_SIZE / 2.0, CHUNK_SIZE / 2.0],
                nr_instances: nr_instances * 50,
                growth_texture_id: 1,
                scale: 1.6,
                height_modifier: 0.6,
            },
            chunk: chunk.clone(),
            distance_culling: DistanceCulling { distance: 300.0 },
            ..default()
        });
        tot_instances_grass += nr_instances * 50;
    }
    info!("Total grass straws {:?}", tot_instances_grass);

    // Camera
    commands
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            x_angle: 45.0_f32.to_radians(),
            y_angle: 45.0_f32.to_radians(),
            max_center: Vec3::splat(1000.0), //Assume square map
            min_center: Vec3::splat(-1000.0),
            distance: 5.0,
            max_distance: 100.0,
            pan_sensitivity: 1.5,
            max_y_angle: 80.0_f32.to_radians(),
            min_y_angle: 5.0_f32.to_radians(),
            ..Default::default()
        })
        .insert(Name::new("Camera"));
}

fn setup_plants(
    mut commands: Commands,
    gltf_meshes: Res<Assets<GltfMesh>>,
    assets_gltf: Res<Assets<Gltf>>,
    my_gltf_assets: Res<MyGltfAssets>,
    materials: Res<Assets<StandardMaterial>>,
) {
    //Load all models and textures (There has to be a better way than this?)
    let mushroom_gltf = assets_gltf.get(&my_gltf_assets.mushroom).unwrap();
    let mushroom_primitive = &gltf_meshes
        .get(&mushroom_gltf.meshes[0])
        .unwrap()
        .primitives[0];
    let mushroom_mesh_handle = mushroom_primitive.mesh.clone();
    let mushroom_texture = materials
        .get(&mushroom_primitive.material.clone().unwrap())
        .unwrap()
        .base_color_texture
        .clone()
        .unwrap();

    let tree_gltf = assets_gltf.get(&my_gltf_assets.tree).unwrap();
    let tree_primitive = &gltf_meshes.get(&tree_gltf.meshes[0]).unwrap().primitives[0];
    let tree_mesh_handle = tree_primitive.mesh.clone();
    let tree_texture = materials
        .get(&tree_primitive.material.clone().unwrap())
        .unwrap()
        .base_color_texture
        .clone()
        .unwrap();

    let rock_gltf = assets_gltf.get(&my_gltf_assets.rock).unwrap();
    let rock_primitive = &gltf_meshes.get(&rock_gltf.meshes[0]).unwrap().primitives[0];
    let rock_mesh_handle = rock_primitive.mesh.clone();
    let rock_texture = materials
        .get(&rock_primitive.material.clone().unwrap())
        .unwrap()
        .base_color_texture
        .clone()
        .unwrap();

    let bush_gltf = assets_gltf.get(&my_gltf_assets.bush).unwrap();
    let bush_primitive = &gltf_meshes.get(&bush_gltf.meshes[0]).unwrap().primitives[0];
    let bush_mesh_handle = bush_primitive.mesh.clone();
    let bush_texture = materials
        .get(&bush_primitive.material.clone().unwrap())
        .unwrap()
        .base_color_texture
        .clone()
        .unwrap();

    let nr_instances = (CHUNK_SIZE * CHUNK_SIZE * INSTANCE_DENSITY as f32) as u32;
    let mut tot_instances = 0;
    for (chunk_x, chunk_y) in (0..NR_SIDE_CHUNKS).cartesian_product(0..NR_SIDE_CHUNKS) {
        let chunk_x_pos = chunk_x as f32 * CHUNK_SIZE - CHUNK_SIZE * NR_SIDE_CHUNKS as f32 / 2.0;
        let chunk_y_pos = chunk_y as f32 * CHUNK_SIZE - CHUNK_SIZE * NR_SIDE_CHUNKS as f32 / 2.0;
        let chunk = Chunk {
            chunk_xy: [chunk_x, chunk_y],
        };

        commands.spawn(ChunkInstancingBundle {
            transform: Transform::from_xyz(chunk_x_pos, chunk_y_pos, 0.0),
            mesh_handle: mushroom_mesh_handle.clone(),
            aabb: Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(CHUNK_SIZE, CHUNK_SIZE, 0.0), //Why do I need full chunk_size here?!
            },
            chunk_instancing: ChunkInstancing::new(
                nr_instances / 5,
                mushroom_texture.clone(),
                Transform::from_rotation(Quat::from_rotation_x(90_f32.to_radians()))
                    .with_scale(Vec3::splat(0.05)),
                CHUNK_SIZE,
            ),
            chunk: chunk.clone(),
            distance_culling: DistanceCulling { distance: 100.0 },
            ..default()
        });
        tot_instances += nr_instances / 5;

        commands.spawn(ChunkInstancingBundle {
            transform: Transform::from_xyz(chunk_x_pos, chunk_y_pos, 0.0),
            mesh_handle: tree_mesh_handle.clone(),
            aabb: Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(CHUNK_SIZE, CHUNK_SIZE, 0.0), //Why do I need full chunk_size here?!
            },
            chunk_instancing: ChunkInstancing::new(
                nr_instances / 15,
                tree_texture.clone(),
                Transform::from_rotation(Quat::from_rotation_x(0_f32.to_radians()))
                    .with_scale(Vec3::splat(0.2)),
                CHUNK_SIZE,
            ),
            chunk: chunk.clone(),
            distance_culling: DistanceCulling { distance: 600.0 },
            ..default()
        });
        tot_instances += nr_instances / 15;

        commands.spawn(ChunkInstancingBundle {
            transform: Transform::from_xyz(chunk_x_pos, chunk_y_pos, 0.0),
            mesh_handle: bush_mesh_handle.clone(),
            aabb: Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(CHUNK_SIZE, CHUNK_SIZE, 0.0), //Why do I need full chunk_size here?!
            },
            chunk_instancing: ChunkInstancing::new(
                nr_instances / 6,
                bush_texture.clone(),
                Transform::from_rotation(Quat::from_rotation_x(0_f32.to_radians()))
                    .with_scale(Vec3::splat(0.4)),
                CHUNK_SIZE,
            ),
            chunk: chunk.clone(),
            distance_culling: DistanceCulling { distance: 200.0 },
            ..default()
        });
        tot_instances += nr_instances / 6;

        commands.spawn(ChunkInstancingBundle {
            transform: Transform::from_xyz(chunk_x_pos, chunk_y_pos, 0.0),
            mesh_handle: rock_mesh_handle.clone(),
            aabb: Aabb {
                center: Vec3A::ZERO,
                half_extents: Vec3A::new(CHUNK_SIZE, CHUNK_SIZE, 0.0), //Why do I need full chunk_size here?!
            },
            chunk_instancing: ChunkInstancing::new(
                nr_instances / 10,
                rock_texture.clone(),
                Transform::from_rotation(Quat::from_rotation_x(0_f32.to_radians()))
                    .with_scale(Vec3::splat(0.6)),
                CHUNK_SIZE,
            ),
            chunk: chunk.clone(),
            distance_culling: DistanceCulling { distance: 200.0 },
            ..default()
        });
        tot_instances += nr_instances / 10;
    }
    info!("Total instanced objects {:?}", tot_instances);
}
