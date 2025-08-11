use crate::chunks::generate::generate_chunk;

// todo: player access chunks - 3*3*3 chunks area
// player access meshes - 16*16*16 chunks area
// player position -> scanner chunk position 

use super::math::*;
use spacetimedb::{
    table, ReducerContext, Table,
    //client_visibility_filter, Filter
};
use std::collections::*;
use include_directory::{include_directory, Dir};

pub mod blocks;
mod generate;

pub use blocks::*;

pub(super) static SCHEME_DIR: Dir<'_> = include_directory!("./scheme");

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

    let blocks_file = SCHEME_DIR.get_file("blocks.json")
        .expect("Blocks data file is not found");
    
    let blocks: Vec<(String, ModelType)> = blocks_file.contents_utf8()
        .and_then(|data| serde_json::from_str(data).ok())
        .expect("Blocks data file parse error");

    for (id, (name, model)) in blocks.into_iter().enumerate() {
        let id = id as u16;
        ctx.db.block().insert(Block { id, name, model });
    }
}

/// Generate world area
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