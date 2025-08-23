use super::*;
use crate::chunks::config::{WorldConfig, build_noise_bank, get_noise_layer};
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
    let blocks = Chunk::empty();
    let mut chunk = Chunk::new(pos.into(), blocks);

    let cfg = WorldConfig::load(ctx);
    let range = cfg.chunk_range;

    if pos.x > range
        || pos.x < -range
        || pos.y > cfg.chunk_height_range
        || pos.y < cfg.chunk_bottom_range
        || pos.z > range
        || pos.z < -range
    {
        return ctx
            .db
            .chunk()
            .insert(Chunk::new(pos.into(), Chunk::empty()));
    }

    let bank = build_noise_bank(ctx);

    let base = get_noise_layer(&bank, "base").expect("No 'base' noise configured");
    let mountains =
        get_noise_layer(&bank, "mountain_height").expect("No 'mountain_height' noise configured");
    let mountain_mask =
        get_noise_layer(&bank, "mountain_mask").expect("No 'mountain_mask' noise configured");

    let id_grass = find_block(ctx, "grass");
    let id_dirt = find_block(ctx, "dirt");
    let id_stone = find_block(ctx, "stone");
    let id_error_block = find_block(ctx, "error_pyrple");

    for x in 0..SIZE_I32 {
        for z in 0..SIZE_I32 {
            let world_x = x + pos.x * SIZE_I32;
            let world_z = z + pos.z * SIZE_I32;

            // Горный шум
            let n_mtn = mountains.noise.sample2d(world_x as f32, world_z as f32);
            let h_mtn = (mountains.base_level_blocks - 64) as f32
                + ((n_mtn * mountains.amplitude_blocks as f32).max(0.0));

            // Маска для редкости гор
            let mask_val = mountain_mask.noise.sample2d(world_x as f32, world_z as f32);

            // Базовая поверхность для равнин
            let n_base = base.noise.sample2d(world_x as f32, world_z as f32);
            let h_base =
                (base.base_level_blocks as f32 + n_base * base.amplitude_blocks as f32).round();

            for y_local in 0..SIZE_I32 {
                let world_y = pos.y * SIZE_I32 + y_local;
                let index = Chunk::block_index(IVec3::new(x, y_local, z));
                if (index as usize) >= chunk.blocks.len() {
                    continue;
                }

                // Нижние чанки полностью камень
                if world_y < -SIZE_I32 {
                    chunk.blocks[index] = id_stone;
                    continue;
                }

                // Горы
                if mask_val > 0.135 && world_y >= -2 * SIZE_I32 && (world_y as f32) <= h_mtn {
                    let dy = h_mtn - world_y as f32;
                    if dy > 0.0 {
                        let threshold = (dy / 3.0).powf(10.0); // острые пики
                        if threshold > 0.5 && chunk.blocks[index] == 0 {
                            chunk.blocks[index] = id_stone;
                        }
                    }
                }

                // Равнины
                if (world_y as f32) <= h_base && chunk.blocks[index] == 0 {
                    if world_y as f32 == h_base {
                        chunk.blocks[index] = id_grass;
                    } else if world_y as f32 >= h_base - 2.0 {
                        chunk.blocks[index] = id_dirt;
                    } else {
                        chunk.blocks[index] = id_stone;
                    }
                }
            }
        }
    }

    ctx.db.chunk().insert(chunk)
}
