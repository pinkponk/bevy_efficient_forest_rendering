
#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions

#import bevy_pbr::pbr_bindings

#import bevy_pbr::utils
#import bevy_pbr::clustered_forward
#import bevy_pbr::lighting
#import bevy_pbr::shadows
#import bevy_pbr::pbr_functions

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,

};

struct InstanceInput {
    @location(3) xyz: vec4<f32>,
}

struct PlantChunk{
    model_transform: mat4x4<f32>
}

 @group(2) @binding(0)
 var<uniform> plant_chunk: PlantChunk;


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};



// [0,1.0]
fn rand(co: vec2<f32>, seed: f32)-> f32{
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

@vertex
fn vertex(vertex: Vertex,
    instance: InstanceInput,
) -> VertexOutput {

    var out: VertexOutput;
    out.uv = vertex.uv;

    let rand_scale = rand(vec2<f32>(instance.xyz.y, 42.546*sin(instance.xyz.x)), 3.0)*0.2+0.9;
    let transformed_position = plant_chunk.model_transform*vec4<f32>(vertex.position, 1.0)*instance.xyz.w*rand_scale;

    let rot_z = rand(vec2<f32>(instance.xyz.x, 10.1512515*cos(instance.xyz.y)), 1.0)*3.1415*2.0;

    let rot_mat = mat2x2<f32>(vec2<f32>(cos(rot_z), -sin(rot_z)), vec2<f32>(sin(rot_z), cos(rot_z)));
    let rotated_xy = rot_mat*transformed_position.xy;
    let position= vec4<f32>(rotated_xy.x+instance.xyz.x, rotated_xy.y+instance.xyz.y, transformed_position.z+instance.xyz.z, 1.0);

    //Somthing not right about the normals?
    let transformed_normals = plant_chunk.model_transform*vec4<f32>(vertex.normal, 1.0);
    let rotated_normals = rot_mat*transformed_normals.xy;
    let normals= vec3<f32>(rotated_normals.x,rotated_normals.y,transformed_normals.z);

    out.world_position = mesh_position_local_to_world(mesh.model, position);
    out.world_normal = mesh_normal_local_to_world(normals);
    out.clip_position = mesh_position_world_to_clip(out.world_position);
    return out;
}

@group(3) @binding(0)
var diffuse_texture: texture_2d<f32>;
@group(3) @binding(1)
var diffuse_sampler: sampler;

// @fragment
// fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
//     return textureSample(diffuse_texture, diffuse_sampler, in.uv);
//     // return  vec4<f32>(0.5,0.5,0.5,1.0);
// }




struct FragmentInput {
    @builtin(front_facing) is_front: bool,
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // Prepare a 'processed' StandardMaterial by sampling all textures to resolve
    // the material members
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = textureSample(diffuse_texture, diffuse_sampler, in.uv);
    pbr_input.material.reflectance = 0.0;
    // pbr_input.material.emissive = 0.0;

    pbr_input.frag_coord = in.frag_coord;
    pbr_input.world_position = in.world_position;
    pbr_input.world_normal = in.world_normal;

    pbr_input.is_orthographic = view.projection[3].w == 1.0;

    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        pbr_input.world_normal,
        in.uv,
    );
    pbr_input.V = calculate_view(in.world_position, pbr_input.is_orthographic);

    return pbr(pbr_input);
}
