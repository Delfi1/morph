use crate::chunks::chunk;

use super::{
    math::*,
    chunks::Chunk
};
use spacetimedb::{
    reducer, table, ReducerContext,
    ScheduleAt, Table, TimeDuration
};

// todo: mesh lods

#[table(name = mesh, public)]
/// Mesh table (or cached mesh)
pub struct Mesh {
    #[unique]
    position: StIVec3,
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

impl Mesh {
    /// Mesh builder
    pub fn build(ctx: &ReducerContext, chunk: &Chunk) -> Mesh {
        todo!();
    }
}

#[table(name = mesher_schedule, scheduled(run_mesher))]
pub struct MeshBuildSchedule {
    #[primary_key]
    #[auto_inc]
    pub scheduled_id: u64,
    pub scheduled_at: ScheduleAt,
    
    #[unique]
    pub position: StIVec3
}

// todo get nearby chunks array
fn get_chunk(ctx: &ReducerContext, position: &StIVec3) -> Option<Chunk> {
    ctx.db.chunk().iter().find(|chunk| chunk.position == *position)
}

#[reducer]
fn run_mesher(ctx: &ReducerContext, arg: MeshBuildSchedule) -> Result<(), String> {
    if ctx.sender != ctx.identity() {
        return Err("Mesher may not be invoked by clients, only via scheduling.".into());
    }

    let Some(chunk) = get_chunk(ctx, &arg.position) else {
        ctx.db.mesher_schedule().scheduled_id().delete(&arg.scheduled_id);
        
        run_mesh_task(ctx, arg);
        return Ok(());
    };

    todo!();
}

pub fn run_mesh_task(ctx: &ReducerContext, mut arg: MeshBuildSchedule) {
    let delay = TimeDuration::from_micros(15_000);

    arg.scheduled_at = (ctx.timestamp + delay).into();
    ctx.db.mesher_schedule().insert(arg);
}