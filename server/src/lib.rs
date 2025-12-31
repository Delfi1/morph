use spacetimedb::{Identity, ReducerContext, Table, Timestamp};

#[spacetimedb::table(name = player)]
pub struct Player {
    #[auto_inc]
    #[primary_key]
    id: u64,
}

/// Chunk data
pub struct MChunk {
    px: i32,
    py: i32,
    pz: i32
}

#[spacetimedb::reducer(init)]
fn init(_ctx: &ReducerContext) {
    shared::init();

    
}
