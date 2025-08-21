// worldgen

use spacetimedb::rand::Rng;
use super::*;

/// World gen context
/// todo: load data from config
pub struct _Generator {
    
}

pub fn block_by_name(ctx: &ReducerContext, name: impl Into<String>) -> u16 {
    ctx.db.block().name().find(&name.into())
        .and_then(|b| Some(b.id)).unwrap_or(0)
}

pub fn generate_chunk(ctx: &ReducerContext, pos: IVec3) -> Option<Chunk> {
    // WIP: dynamic world size
    let range = 4;
    if pos.x > range || pos.x < -range || pos.y > range || pos.y < -range || pos.z > range || pos.z < -range {
        return None;
    }

    let mut blocks = Chunk::empty();

    let vals = ["air", "dirt", "grass"];
    let l = vals.len();
        
    if pos.y == 0 {
        for i in 0..SIZE.pow(2) {
            let rand = ctx.rng().gen_range(0..l);

            blocks[i] = block_by_name(ctx, vals[rand]);
        }
    }
    
    Some(ctx.db.chunk().insert(Chunk::new(pos.into(), blocks)))
}