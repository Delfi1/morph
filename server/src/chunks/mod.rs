use crate::chunks::generate::generate_chunk;

// FIXME: optimize chunks size

// todo: player access chunks - 3*3*3 chunks area
// player access meshes - 16*16*16 chunks area
// player position -> scanner chunk position 

use super::math::*;
use spacetimedb::{
    reducer, table, ReducerContext,
    ScheduleAt, Table, TimeDuration
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
    pub position: StIVec3,
    // Vec of blocks
    pub blocks: Vec<u16>
}

impl Chunk {
    pub fn empty() -> Vec<u16> {
        return std::iter::repeat_n(0, SIZE_P3).collect();
    }

    pub fn new(position: StIVec3, blocks: Vec<u16>) -> Self {
        Self { position, blocks }
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

#[table(name = chunk_schedule, scheduled(run_generator))]
pub struct ChunkSchedule {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
    
    #[unique]
    pub position: StIVec3
}

#[reducer]
fn run_generator(ctx: &ReducerContext, arg: ChunkSchedule) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        return Err("Generator may not be invoked by clients, only via scheduling.".into());
    }

    generate_chunk(ctx, arg.position.into());

    Ok(())
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

    let delay = TimeDuration::from_micros(15_000);
    let scheduled_at = (ctx.timestamp + delay).into();
    for pos in area {
        ctx.db.chunk_schedule().insert(ChunkSchedule {
            scheduled_id: 0,
            scheduled_at,
            position: pos.into()
        });
    }
}