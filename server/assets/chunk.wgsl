struct CameraUniform {
    // Camera projection
    clip_from_view: mat4x4<f32>,
    // Camera view
    view_from_world: mat4x4<f32>,
}

struct ChunkUniform {
    // Chunk transform in chunk grid
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;

@group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
@group(1) @binding(1) var nearest_sampler: sampler;

@group(2) @binding(0) var<uniform> chunk: ChunkUniform;


// Packed voxel data
struct Vertex {
    @location(0) data: u32,
};

// Unpack bits mask
fn x_bits(bits: u32) -> u32{
    return (1u << bits) - 1u;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    // Uv coords in texture
    @location(1) uv: vec2<f32>, 
    // block id (or model uniform id)
    @location(2) block: u32,
    @location(3) side: u32,
}

var<private> normals: array<vec3<f32>, 6> = array<vec3<f32>,6> (
	vec3<f32>(0.0, 1.0, 0.0),   // Up
    vec3<f32>(-1.0, 0.0, 0.0),  // Left
	vec3<f32>(1.0, 0.0, 0.0),   // Right
	vec3<f32>(0.0, 0.0, -1.0),  // Forward
	vec3<f32>(0.0, 0.0, 1.0),   // Back
    vec3<f32>(0.0, -1.0, 0.0),  // Down
);

// Cube model uv map
// Uv map (x0; y0; x1; y1)
var<private> cube: array<vec4<f32>, 6> = array<vec4<f32>, 6>(
    vec4<f32>(0.625, 1.0, 0.375, 0.75),  // Up
    vec4<f32>(0.625, 0.0, 0.875, 0.25),  // Left
    vec4<f32>(0.125, 0.0, 0.375, 0.25),  // Right
    vec4<f32>(0.625, 0.0, 0.375, 0.25),  // Forward
    vec4<f32>(0.625, 0.75, 0.375, 0.5),  // Back
    vec4<f32>(0.625, 0.5, 0.375, 0.25),  // Down
);

// Get uv from block type, side and uv(xy) 
fn get_uv(block_type: u32, side: u32, x: u32, y: u32) -> vec2<f32> {
    let idx = x * 2u;
    let idy = y * 2u + 1u;

    return vec2<f32>(cube[side][idx], cube[side][idy]);
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    // Unpack vertex data
    let x = f32(vertex.data & x_bits(6u));
    let y = f32((vertex.data >> 6u) & x_bits(6u));
    let z = f32((vertex.data >> 12u) & x_bits(6u));
    
    // Side (also normal index)
    let side = (vertex.data >> 18u) & x_bits(3u);

    let uvx = (vertex.data >> 21u) & x_bits(1u);
    let uvy = (vertex.data >> 22u) & x_bits(1u);

    // Block type
    let block_type = (vertex.data >> 23u) & x_bits(2u);

    // Block id (also model and texture id)
    let block = (vertex.data >> 25u) & x_bits(7u);

    out.position = camera.clip_from_view * camera.view_from_world * chunk.transform * vec4<f32>(x, y, z, 1.0);
    out.normal = normals[side];
    out.uv = get_uv(block_type, side, uvx, uvy);
    out.block = block;
    out.side = side;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color: vec4<f32> = textureSample(textures[in.block], nearest_sampler, in.uv);

    // todo: add direction light

    return color;
}