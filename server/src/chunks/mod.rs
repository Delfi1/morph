use crate::chunks::generate::generate_chunk;

use super::math::*;
use spacetimedb::{
    table, ReducerContext, Table,
    client_visibility_filter, Filter
};
use std::collections::*;

pub mod blocks;
mod generate;

pub use blocks::*;

pub const SIZE: usize = 32;
pub const SIZE_P3: usize = SIZE.pow(3);

// Collection of blocks
#[table(name = chunk, public)]
pub struct Chunk {
    #[unique]
    position: StIVec3,
    // Data of blocks Vec<u16> -> bincode
    data: Vec<u8>
}

impl Chunk {
    pub fn empty() -> Vec<u16> {
        return std::iter::repeat_n(0, SIZE_P3).collect();
    }

    pub fn new(position: StIVec3, blocks: Vec<u16>) -> Self {
        let data = bincode::encode_to_vec(
            blocks, bincode::config::standard()
        ).unwrap();

        Self { position, data }
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

pub fn generate(ctx: &ReducerContext, range: usize) {
    let mut area = HashSet::with_capacity(range.pow(3));

    let range = range as i32;
    for x in -range..=range {
        for y in -range..=range {
            for z in -range..=range {
                area.insert(ivec3(x, y, z));
            }
        }
    }

    for pos in area {
        generate_chunk(ctx, pos);
    }
}