use super::*;
use crate::chunks::config::{WorldConfig, get_noise, load_noise_config};
use spacetimedb::ReducerContext;

pub struct _Generator {}

pub fn find_block(ctx: &ReducerContext, name: impl Into<String>) -> u16 {
    ctx.db
        .block()
        .name()
        .find(name.into())
        .map(|b| b.id)
        .unwrap_or(0)
}

pub fn generate_chunk(ctx: &ReducerContext, pos: IVec3) -> Chunk {
    let mut blocks = Chunk::empty();
    let cfg = WorldConfig::load(ctx);
    let noise_cfg = load_noise_config(ctx);

    let range = cfg.chunk_range;
    if pos.x > range
        || pos.x < -range
        || pos.y > range
        || pos.y < -range
        || pos.z > range
        || pos.z < -range
    {
        return ctx.db.chunk().insert(Chunk::new(pos.into(), blocks));
    }

    if pos.y == 0 {
        let noise = get_noise(ctx);
        let base = noise_cfg.base_level_blocks;
        let amp = noise_cfg.amplitude_blocks;

        for x in 0..SIZE_I32 {
            for z in 0..SIZE_I32 {
                let world_x = x as f32 + (pos.x * SIZE_I32) as f32;
                let world_z = z as f32 + (pos.z * SIZE_I32) as f32;
                let n = noise.sample2d(world_x, world_z);
                let height = (base + (n * amp as f32) as i32).clamp(0, SIZE_I32 - 1);

                for y in 0..SIZE_I32 as i32 {
                    if y < height {
                        let bpos = IVec3::new(x as i32, y, z as i32);
                        let index = Chunk::block_index(bpos);
                        if (index as usize) < blocks.len() {
                            blocks[index] = if height - 1 <= y {
                                find_block(ctx, "grass")
                            } else if height - 3 <= y {
                                find_block(ctx, "dirt")
                            } else {
                                find_block(ctx, "stone")
                            };
                        }
                    }
                }
            }
        }
    }

    ctx.db.chunk().insert(Chunk::new(pos.into(), blocks))
}
