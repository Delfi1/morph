use super::math::*;
use spacetimedb::{table, SpacetimeType};

#[derive(SpacetimeType)]
// Model type with texture file id
pub enum BlockModel {
    Cube(u64),
    Stair(u64),
    Slab(u64),
    Custom(u64)
}

// Block type table
#[table(name=block)]
pub struct Block {
    #[auto_inc]
    #[primary_key]
    id: u64,
    // Texture path
    model: BlockModel,
}

// Collection of blocks
#[table(name = chunk, public)]
pub struct Chunk {
    #[unique]
    position: StIVec3,
    // Data of block ids
    blocks: Vec<u64>
}

