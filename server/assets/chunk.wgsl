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

/// Block model types by block id
@group(2) @binding(0) var<storage, read> blocks: array<u32>;

@group(3) @binding(0) var<uniform> chunk: ChunkUniform;


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
    // block id (or model uniform id)
    @location(0) block: u32,
    @location(1) side: u32,
    // Uv coords in texture
    @location(2) uv: vec2<f32>, 
}

var<private> light: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

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
    vec4<f32>(0.0, 0.0, 0.1666, 1.0),   // Up
    vec4<f32>(0.1666, 0.0, 0.333, 1.0), // Left
    vec4<f32>(0.333, 0.0, 0.5, 1.0),    // Right
    vec4<f32>(0.5, 0.0, 0.666, 1.0),    // Forward
    vec4<f32>(0.666, 0.0, 0.8333, 1.0), // Back
    vec4<f32>(0.8333, 0.0, 1.0, 1.0),   // Down
);

// Get uv from block type, side and uv(xy) 
fn get_uv(block_type: u32, side: u32, uvx: u32, uvy: u32) -> vec2<f32> {
    // if block_type is 1u -> cube; 2u -> slab; 3u -> stairs;

    let idx = uvx * 2u;
    let idy = uvy * 2u + 1u;

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

    // Block id (also model and texture id)
    let block = (vertex.data >> 23u) & x_bits(9u);
    let block_type = blocks[block];

    out.position = camera.clip_from_view * camera.view_from_world * chunk.transform * vec4<f32>(x, y, z, 1.0);
    out.block = block;
    out.side = side;
    out.uv = get_uv(block_type, side, uvx, uvy);
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    //let block_type = blocks[in.block];
    let normal = normals[in.side];
    let color: vec4<f32> = textureSample(textures[in.block], nearest_sampler, in.uv);

    // todo: add direction light

    return color;
}