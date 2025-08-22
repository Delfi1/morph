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

use bevy_math::IVec3;
use std::path::Path;

pub static HEIGHTMAP: OnceCell<Vec<Vec<u8>>> = OnceCell::new();

/// Загружает PNG карту высот 32x32
pub fn load_heightmap() -> &'static Vec<Vec<u8>> {
    HEIGHTMAP.get_or_init(|| {
        let path = Path::new("assets/perlin_32_2.png");
        let img = ImageReader::open(path)
            .expect("Failed to open heightmap PNG")
            .decode()
            .expect("Failed to decode PNG");

        let (width, height) = img.dimensions();
        let mut map = vec![vec![0u8; width as usize]; height as usize];

        for y in 0..height {
            for x in 0..width {
                let pixel = img.get_pixel(x, y);
                // используем красный канал как высоту
                map[y as usize][x as usize] = pixel[0];
            }
        }

        map
    })
}

/// Возвращает высоту для координат (x, z)
pub fn height_at(x: usize, z: usize) -> u8 {
    let map = load_heightmap();
    let w = map[0].len();
    let h = map.len();

    let x = x % w;
    let z = z % h;

    map[z][x]
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

    let img = load_asset_image(ctx, "perlin_32.png").expect("Нет ассета perlin_32.png");
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

                        // Генерируем блок по высоте
                        blocks[index] = if y < 15 {
                            find_block(ctx, "stone")
                        } else if y < 19 {
                            find_block(ctx, "dirt")
                        } else {
                            find_block(ctx, "grass")
                        };
                    }
                }
            }
        }
    }

    ctx.db.chunk().insert(Chunk::new(pos.into(), blocks))
}
