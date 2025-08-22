use include_directory::{Dir, include_directory};

use image::GenericImageView;

use image::{GrayImage, Luma};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// Функция сглаживания
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

// Линейная интерполяция
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

// Градиенты для 2D
fn grad(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 3 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

fn perlin(x: f64, y: f64, perm: &[u8; 256]) -> f64 {
    let xi = x.floor() as usize & 255;
    let yi = y.floor() as usize & 255;

    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let xi1 = (xi + 1) & 255;
    let yi1 = (yi + 1) & 255;

    let aa = perm[(perm[xi] as usize + yi) & 255];
    let ab = perm[(perm[xi] as usize + yi1) & 255];
    let ba = perm[(perm[xi1] as usize + yi) & 255];
    let bb = perm[(perm[xi1] as usize + yi1) & 255];

    let x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad(ab, xf, yf - 1.0), grad(bb, xf - 1.0, yf - 1.0), u);

    lerp(x1, x2, v)
}

fn perlin_fbm(x: f64, y: f64, perm: &[u8; 256], octaves: u32, persistence: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin(x * frequency, y * frequency, perm) * amplitude;
        max_value += amplitude;

        amplitude *= persistence; // уменьшение влияния каждой следующей октавы
        frequency *= 2.0; // увеличение частоты (мельче детали)
    }

    total / max_value // нормализация в [-1,1]
}

fn perlin_creat() {
    let width = 32;
    let height = 32;
    let scale = 0.016;
    let octaves = 4;
    let persistence = 0.48;
    let seed = 164468418;

    let mut prefiks: String = String::from("_32_test.png");
    let mut png_name: String = String::from("perlin");
    png_name = png_name + &prefiks;

    // Генератор случайных чисел
    let mut rng = StdRng::seed_from_u64(seed);
    let mut perm = [0u8; 256];
    for i in 0..256 {
        perm[i] = i as u8;
    }
    for i in (1..256).rev() {
        let j = rng.random_range(0..=i);
        perm.swap(i, j);
    }

    // Создание изображения
    let mut img = GrayImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let value = perlin_fbm(
                x as f64 * scale,
                y as f64 * scale,
                &perm,
                octaves,
                persistence,
            );
            let pixel = ((value + 1.0) * 0.5 * 255.0) as u8;
            img.put_pixel(x, y, Luma([pixel]));
        }
    }

    img.save(&png_name).unwrap();

    let img = image::open(png_name).unwrap();

    let (width, height) = img.dimensions();
    let max_height = 3;

    //let mut world = vec![vec![vec![0u8; height as usize]; max_height]; width as usize];

    /*for x in 0..width {
        for y in 0..height {
            // Берём цвет пикселя
            let pixel = img.get_pixel(x, y);

            // Берём яркость (канал R, можно усреднить RGB)
            let brightness = pixel[0] as f64 / 255.0;

            // Преобразуем яркость в высоту (z)
            let z_max = (brightness * max_height as f64) as usize;

            // Заполняем блоки от 0 до z_max
            for z in 0..=z_max {
               // world[x as usize][z][y as usize] = 1; // 1 = блок земли/каменя

            }
        }
    }*/
}
