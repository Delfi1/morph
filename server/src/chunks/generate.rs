// worldgen

use spacetimedb::rand::Rng;
use super::*;

/// World gen context
/// todo: load data from config
pub struct _Generator {
    // todo
}

// Get block by name
pub fn find_block(ctx: &ReducerContext, name: impl Into<String>) -> u16 {
    ctx.db.block().name().find(&name.into())
        .and_then(|b| Some(b.id)).unwrap_or(0)
}

pub fn generate_chunk(ctx: &ReducerContext, pos: IVec3) -> Chunk {
    // WIP: dynamic world size
    let mut blocks = Chunk::empty();

    let range = 4;
    if pos.x > range || pos.x < -range || pos.y > range || pos.y < -range || pos.z > range || pos.z < -range {
        return ctx.db.chunk().insert(Chunk::new(pos.into(), blocks));
    }

    let vals = ["air", "dirt", "grass", "stone"];
    let l = vals.len();

    if pos.y == 0 {
        for i in 0..SIZE.pow(3)/2 {
            let rand = ctx.rng().gen_range(0..l);

            blocks[i] = find_block(ctx, vals[rand]);
        }
    }
    
    ctx.db.chunk().insert(Chunk::new(pos.into(), blocks))
}