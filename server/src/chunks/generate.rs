// worldgen

use super::*;

use spacetimedb::rand::Rng;

use image::{DynamicImage, GenericImageView, load_from_memory};
use image::{GrayImage, io::Reader as ImageReader};
use once_cell::sync::OnceCell;

use crate::assets::load_asset_image;

/// World gen context
/// todo: load data from config
pub struct _Generator {
    // todo
}

// Get block by name
pub fn find_block(ctx: &ReducerContext, name: impl Into<String>) -> u16 {
    ctx.db
        .block()
        .name()
        .find(&name.into())
        .and_then(|b| Some(b.id))
        .unwrap_or(0)
}

pub fn generate_chunk(ctx: &ReducerContext, pos: IVec3) -> Chunk {
    let mut blocks = Chunk::empty();

    let range = 2;
    if pos.x > range
        || pos.x < -range
        || pos.y > range
        || pos.y < -range
        || pos.z > range
        || pos.z < -range
    {
        return ctx.db.chunk().insert(Chunk::new(pos.into(), blocks));
    }

    let img = load_asset_image(ctx, "perlin_32_2.png").expect("Нет ассета perlin.png");
    if pos.y == 0 {
        for x in 0..SIZE {
            for z in 0..SIZE {
                // Берём пиксель по горизонтальной плоскости Z,Y
                let pixel = img.get_pixel(z as u32, x as u32); // z → ширина, x → высота
                let brightness = pixel[0] as f64 / 255.0; // нормализуем яркость
                let height = (brightness * SIZE as f64) as i32;

                for y in 0..SIZE as i32 {
                    if y < height {
                        let pos = IVec3::new(x as i32, y, z as i32);
                        let index = Chunk::block_index(pos);

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

    ctx.db.chunk().insert(Chunk::new(pos.into(), blocks))
}
