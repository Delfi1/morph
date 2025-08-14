// worldgen

use super::*;

/// World gen context
/// todo: load data from config
pub struct _Generator {
    
}

pub fn block_by_name(ctx: &ReducerContext, name: impl Into<String>) -> u16 {
    let name = name.into();
    ctx.db.block().iter().find(|b| &b.name == &name)
        .and_then(|b| Some(b.id)).unwrap_or(0)
}

pub fn generate_chunk(ctx: &ReducerContext, pos: IVec3) -> Option<Chunk> {
    // WIP: dynamic world size
    let range = 4;
    if pos.x > range || pos.x < -range || pos.y > range || pos.y < -range || pos.z > range || pos.z < -range {
        return None;
    }

    let mut blocks = Chunk::empty();

    if pos.y == 0 {
        for i in 0..SIZE.pow(2) {
            blocks[i] = block_by_name(ctx, "dirt");
        }
    }

    Some(ctx.db.chunk().insert(Chunk::new(pos.into(), blocks)))
}