
#import bevy_pbr::mesh_types
#import bevy_pbr::mesh_view_bindings

@group(1) @binding(0)
var<uniform> mesh: Mesh;

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::mesh_functions



// MIT License. © Stefan Gustavson, Munrocket
// perlin noise, Thank you open source!
fn permute4(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn taylorInvSqrt4(r: vec4<f32>) -> vec4<f32> { return 1.79284291400159 - 0.85373472095314 * r; }
fn fade3(t: vec3<f32>) -> vec3<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlinNoise3(P: vec3<f32>) -> f32 {
  var Pi0 : vec3<f32> = floor(P); // Integer part for indexing
  var Pi1 : vec3<f32> = Pi0 + vec3<f32>(1.); // Integer part + 1
  Pi0 = Pi0 % vec3<f32>(289.);
  Pi1 = Pi1 % vec3<f32>(289.);
  let Pf0 = fract(P); // Fractional part for interpolation
  let Pf1 = Pf0 - vec3<f32>(1.); // Fractional part - 1.
  let ix = vec4<f32>(Pi0.x, Pi1.x, Pi0.x, Pi1.x);
  let iy = vec4<f32>(Pi0.yy, Pi1.yy);
  let iz0 = Pi0.zzzz;
  let iz1 = Pi1.zzzz;

  let ixy = permute4(permute4(ix) + iy);
  let ixy0 = permute4(ixy + iz0);
  let ixy1 = permute4(ixy + iz1);

  var gx0: vec4<f32> = ixy0 / 7.;
  var gy0: vec4<f32> = fract(floor(gx0) / 7.) - 0.5;
  gx0 = fract(gx0);
  var gz0: vec4<f32> = vec4<f32>(0.5) - abs(gx0) - abs(gy0);
  var sz0: vec4<f32> = step(gz0, vec4<f32>(0.));
  gx0 = gx0 + sz0 * (step(vec4<f32>(0.), gx0) - 0.5);
  gy0 = gy0 + sz0 * (step(vec4<f32>(0.), gy0) - 0.5);

  var gx1: vec4<f32> = ixy1 / 7.;
  var gy1: vec4<f32> = fract(floor(gx1) / 7.) - 0.5;
  gx1 = fract(gx1);
  var gz1: vec4<f32> = vec4<f32>(0.5) - abs(gx1) - abs(gy1);
  var sz1: vec4<f32> = step(gz1, vec4<f32>(0.));
  gx1 = gx1 - sz1 * (step(vec4<f32>(0.), gx1) - 0.5);
  gy1 = gy1 - sz1 * (step(vec4<f32>(0.), gy1) - 0.5);

  var g000: vec3<f32> = vec3<f32>(gx0.x, gy0.x, gz0.x);
  var g100: vec3<f32> = vec3<f32>(gx0.y, gy0.y, gz0.y);
  var g010: vec3<f32> = vec3<f32>(gx0.z, gy0.z, gz0.z);
  var g110: vec3<f32> = vec3<f32>(gx0.w, gy0.w, gz0.w);
  var g001: vec3<f32> = vec3<f32>(gx1.x, gy1.x, gz1.x);
  var g101: vec3<f32> = vec3<f32>(gx1.y, gy1.y, gz1.y);
  var g011: vec3<f32> = vec3<f32>(gx1.z, gy1.z, gz1.z);
  var g111: vec3<f32> = vec3<f32>(gx1.w, gy1.w, gz1.w);

  let norm0 = taylorInvSqrt4(
      vec4<f32>(dot(g000, g000), dot(g010, g010), dot(g100, g100), dot(g110, g110)));
  g000 = g000 * norm0.x;
  g010 = g010 * norm0.y;
  g100 = g100 * norm0.z;
  g110 = g110 * norm0.w;
  let norm1 = taylorInvSqrt4(
      vec4<f32>(dot(g001, g001), dot(g011, g011), dot(g101, g101), dot(g111, g111)));
  g001 = g001 * norm1.x;
  g011 = g011 * norm1.y;
  g101 = g101 * norm1.z;
  g111 = g111 * norm1.w;

  let n000 = dot(g000, Pf0);
  let n100 = dot(g100, vec3<f32>(Pf1.x, Pf0.yz));
  let n010 = dot(g010, vec3<f32>(Pf0.x, Pf1.y, Pf0.z));
  let n110 = dot(g110, vec3<f32>(Pf1.xy, Pf0.z));
  let n001 = dot(g001, vec3<f32>(Pf0.xy, Pf1.z));
  let n101 = dot(g101, vec3<f32>(Pf1.x, Pf0.y, Pf1.z));
  let n011 = dot(g011, vec3<f32>(Pf0.x, Pf1.yz));
  let n111 = dot(g111, Pf1);

  var fade_xyz: vec3<f32> = fade3(Pf0);
  let temp = vec4<f32>(f32(fade_xyz.z)); // simplify after chrome bug fix
  let n_z = mix(vec4<f32>(n000, n100, n010, n110), vec4<f32>(n001, n101, n011, n111), temp);
  let n_yz = mix(n_z.xy, n_z.zw, vec2<f32>(f32(fade_xyz.y))); // simplify after chrome bug fix
  let n_xyz = mix(n_yz.x, n_yz.y, fade_xyz.x);
  return 2.2 * n_xyz;
}



 struct GpuGrassMaterial {
    time: f32,

    healthy_tip_color: vec4<f32>,
    healthy_middle_color: vec4<f32>,
    healthy_base_color: vec4<f32>,
    unhealthy_tip_color: vec4<f32>,
    unhealthy_middle_color: vec4<f32>,
    unhealthy_base_color: vec4<f32>,

    chunk_xy: vec2<f32>,
    chunk_half_extents: vec2<f32>,
    growth_texture_id: vec4<i32>,
    height_modifier: vec4<f32>,
    scale_modifier: vec4<f32>
 };

 @group(2) @binding(0)
 var<uniform> material: GpuGrassMaterial;


// struct GrassPool{pool: array<Instance, 3600>}; //Cannot use dynamic array when using Uniform, need Storage for that but Storage is not yet supported on web :()
//  @group(3) @binding(0)
// //  var<storage> grass_pool: GrassPool;
//  var<uniform> grass_pool: GrassPool;


@group(3) @binding(0)
var growth_textures: texture_2d_array<f32>;
@group(3) @binding(1)
var growth_sampler: sampler;



 struct GpuGridConfig {
    grid_center_xy: vec2<f32>, //Assume axis aligned grid otherwise need to calc homogenous coordinate matrix
    grid_half_extents: vec2<f32>,
};

 @group(4) @binding(0)
 var<uniform> grid_config: GpuGridConfig;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,

};



struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(4) color: vec4<f32>,
};


// [0,1.0]
fn rand(co: vec2<f32>)-> f32{
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 438.5453);
}




@vertex
fn vertex(vertex: Vertex,
) -> VertexOutput {
    var out: VertexOutput;


    //Random Base World position
    // let seed = sqrt(material.chunk_xy.x*material.chunk_xy.y+1.1);
    let x = (rand(vec2<f32>(sin(f32(vertex.instance_index)), 1.1512515*cos(f32(vertex.instance_index))))*material.chunk_half_extents.x*2.0);
    let y = (rand(vec2<f32>(0.902415*sin(f32(vertex.instance_index)), cos(f32(vertex.instance_index))))*material.chunk_half_extents.y*2.0);

    let base_position = vec4<f32>(x,y,0.0,1.0);
    let base_position_world = mesh_position_local_to_world(mesh.model, base_position);

    //Random Rotate
    let rot_z = (rand(vec2<f32>(12.554215, f32(vertex.instance_index)))*30.1415);
    let rot_mat = mat2x2<f32>(vec2<f32>(cos(rot_z), -sin(rot_z)), vec2<f32>(sin(rot_z), cos(rot_z)));
    let rotated_xy = rot_mat*vertex.position.xy*material.scale_modifier.x;
    let local_z = vertex.position.z*material.scale_modifier.x*material.height_modifier.x;
    out.world_position= vec4<f32>(rotated_xy.x+base_position_world.x, rotated_xy.y+base_position_world.y, local_z+base_position_world.z, 1.0);
    

    //Growth height adjustments
    let growth_uv = (base_position_world.xy-grid_config.grid_center_xy+grid_config.grid_half_extents)/(grid_config.grid_half_extents*2.0);
    out.uv = growth_uv; // out.uv = vertex.uv;
    let growth = textureSampleLevel(growth_textures,growth_sampler, growth_uv,material.growth_texture_id.x, 0.0).x;
    out.world_position.z = out.world_position.z*growth;

    // "Hide" grass under map (This can be done better, probably by sampling 5x times and adjusting nr_instances based on texture sum over chunk)
    if growth<0.5 && growth/0.5<rand(vec2<f32>(f32(vertex.instance_index), 18.956724*cos(f32(vertex.instance_index)*2.9515))){
        out.world_position.z = out.world_position.z+(-10.0);
        out.clip_position = mesh_position_world_to_clip(out.world_position);
        return out;
    } 

    //Straw distortion
    var scale = 0.1*vertex.position.z*material.height_modifier.x*material.scale_modifier.x;
    var noise_x = (rand(vec2<f32>(x,y+local_z))+(-0.5))*scale;
    var noise_y = (rand(vec2<f32>(x,y+local_z))+(-0.5))*scale;
    out.world_position.x = out.world_position.x+noise_x;
    out.world_position.y = out.world_position.y+noise_y;



    //Wind swing effect
    let time_wave = sin(material.time / 1.0 + out.world_position.x/10.0);
    out.world_position.x = out.world_position.x+time_wave*0.4*out.world_position.z;

    //Grass turbulance effect
    let freq = 2.0;
    let time_wave_x = cos(material.time * freq + base_position.x);
    let time_wave_y = sin(material.time * freq + base_position.y);
    var amp = .2*out.world_position.z;
    var perl_freq = 10.0;
    var perl_noise_x = perlinNoise3(vec3<f32>(base_position.x*perl_freq+time_wave_x, base_position.y*perl_freq, vertex.position.z*perl_freq))*amp;
    var perl_noise_y = perlinNoise3(vec3<f32>(base_position.x*perl_freq, base_position.y*perl_freq+time_wave_y, vertex.position.z*perl_freq))*amp;


    //Grass height noise (Might not be needed)
    var amp = 0.3;
    var freq = 0.2;
    var perl_noise_height = perlinNoise3(vec3<f32>(out.world_position.x*freq, out.world_position.y*freq, vertex.position.z*freq/10.0))*amp*out.world_position.z;

    out.world_position = vec4<f32>(out.world_position.x+perl_noise_x, out.world_position.y+perl_noise_y, out.world_position.z+perl_noise_height, out.world_position.w);


    //Color
    let tip_color = mix(material.unhealthy_tip_color, material.healthy_tip_color, growth);
    let middle_color = mix(material.unhealthy_middle_color, material.healthy_middle_color, growth);
    let base_color = mix(material.unhealthy_base_color, material.healthy_base_color, growth);




    if (vertex.position.z > 0.8){
        out.color =  tip_color;
    } else if (vertex.position.z > 0.3){
        out.color =  middle_color;
    }else{
        out.color =  base_color;
    }

    out.clip_position = mesh_position_world_to_clip(out.world_position);

    return out;
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // return  vec4<f32>(0.5,0.5,0.5,1.0);
    return in.color;
}