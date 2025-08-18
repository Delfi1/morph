
struct ChunkUniform {
    // Chunk position in chunk grid
    position: vec3<f32>,
}

//todo: view matrix and textures
struct DataUniform {
    // Camera projection
    clip_from_view: mat4x4<f32>,
    // Camera transform
    world_from_view: mat4x4<f32>,
}

//@group(0) @binding(0) var<uniform> chunk: ChunkUniform;
//@group(1) @binding(1) var<uniform> data: DataUniform;

const size = 32.0;
const chunk_size = vec3(size, size, size);

struct Vertex {
    // Packed voxel data
    @location(0) data: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    //@location(2) uv: vec2<f32>,
    //@location(3) b: u32,
    //@location(4) side: u32,
}

var<private> normals: array<vec3<f32>, 6> = array<vec3<f32>,6> (
	vec3<f32>(0.0, 1.0, 0.0),   // Up
    vec3<f32>(-1.0, 0.0, 0.0),  // Left
	vec3<f32>(1.0, 0.0, 0.0),   // Right
	vec3<f32>(0.0, 0.0, -1.0),  // Forward
	vec3<f32>(0.0, 0.0, 1.0),   // Back
    vec3<f32>(0.0, -1.0, 0.0),  // Down
);

fn x_bits(bits: u32) -> u32{
    return (1u << bits) - 1u;
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let x = f32(vertex.data & x_bits(6u));
    let y = f32(vertex.data >> 6u & x_bits(6u));
    let z = f32(vertex.data >> 12u & x_bits(6u));
    let normal_index = vertex.data >> 18u & x_bits(3u);
    
    let local_position = vec4<f32>(x, y, z, 1.0) * position;

    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // draw vertices with green color
    return vec4<f32>(0.0, 0.8, 0.0, 1.0)
}