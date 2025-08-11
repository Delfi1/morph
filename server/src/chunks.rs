use super::math::*;
use spacetimedb::{table, ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType)]
// Model type with texture file name
pub enum ModelType {
    Cube(String),
    Stair(String),
    Slab(String),
    Custom(String),
    Empty
}

#[derive(SpacetimeType)]
pub struct BlockType {
    // block type name
    name: String,
    // Texture path
    model: ModelType,
    // light?
    // collision?
}

impl BlockType {
    pub fn new(name: String, model: ModelType) -> Self {
        Self { name, model }
    }
}

// Block type table
#[table(name = block)]
pub struct Block {
    #[primary_key]
    id: u16,
    block_type: BlockType
}

pub const SIZE: usize = 32;
pub const SIZE_P3: usize = SIZE.pow(3);

// Collection of blocks
#[table(name = chunk, public)]
pub struct Chunk {
    #[unique]
    position: StIVec3,
    // Data of block ids
    blocks: Vec<u16>
}

impl Chunk {
    pub fn new(pos: StIVec3) -> Self {
        let blocks = std::iter::repeat_n(0, SIZE_P3).collect();

        Self {
            position: pos,
            blocks: blocks
        }
    }
}

pub fn init_blocks(ctx: &ReducerContext) {
    // clear blocks data
    for block in ctx.db.block().iter() {
        ctx.db.block().id().delete(block.id);
    }

    let types = vec![
        BlockType::new("air".into(), ModelType::Empty),
        BlockType::new("dirt".into(), ModelType::Cube("dirt.png".into())),
        BlockType::new("grass".into(), ModelType::Cube("grass.png".into())),
    ];

    for (id, block_type) in types.into_iter().enumerate() {
        let id = id as u16;
        ctx.db.block().insert(Block { id, block_type });
    }
}