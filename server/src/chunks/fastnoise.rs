//! Minimal FastNoise-like struct for Rust projects
//! - Без зависимостей
//! - Поддерживает Perlin и Value noise (2D)
//! - Поддерживает фрактальный FBM (octaves / lacunarity / gain)
//! - Конфиг через builder-методы
//! - Псевдослучайность из seed через простую xorshift64*
//!
//! Пример использования см. внизу (tests/examples).

#[derive(Clone, Copy, Debug)]
pub enum NoiseType {
    Perlin,
    Value,
}

#[derive(Clone, Copy, Debug)]
pub struct FractalCfg {
    pub octaves: u8,
    pub lacunarity: f32,
    pub gain: f32,
}

#[derive(Clone, Debug)]
pub struct FastNoise {
    seed: u64,
    frequency: f32,
    noise_type: NoiseType,
    fractal: Option<FractalCfg>,
    // Предрасчитанная таблица перестановок для быстрого доступа (0..=255, продублирована)
    p: [u8; 512],
}

impl Default for FastNoise {
    fn default() -> Self {
        Self::new(1337)
    }
}

impl FastNoise {
    pub fn new(seed: u64) -> Self {
        let mut s = Self {
            seed,
            frequency: 0.01, // удобная дефолтная частота
            noise_type: NoiseType::Perlin,
            fractal: None,
            p: [0; 512],
        };
        s.reseed(seed);
        s
    }

    pub fn reseed(&mut self, seed: u64) -> &mut Self {
        self.seed = seed;
        // Cгенерируем перестановку 0..=255, перешафлим xorshift'ом
        let mut base: [u8; 256] = [0; 256];
        for i in 0..256 {
            base[i] = i as u8;
        }
        // простой Fisher–Yates со своим PRNG
        let mut rng = XorShift64Star::new(seed ^ 0x9E3779B97F4A7C15);
        for i in (1..256).rev() {
            // i от 255..1
            let j = (rng.next_u64() as usize) % (i + 1);
            base.swap(i, j);
        }
        // Дублируем в p[512]
        for i in 0..512 {
            self.p[i] = base[i & 255];
        }
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.reseed(seed);
        self
    }
    pub fn with_frequency(mut self, freq: f32) -> Self {
        self.frequency = freq.max(1e-6);
        self
    }
    pub fn with_noise_type(mut self, ty: NoiseType) -> Self {
        self.noise_type = ty;
        self
    }
    pub fn with_fractal(mut self, cfg: FractalCfg) -> Self {
        self.fractal = Some(cfg);
        self
    }
    pub fn without_fractal(mut self) -> Self {
        self.fractal = None;
        self
    }

    /// Главный метод семплинга 2D. Возвращает значение в диапазоне примерно [-1, 1].
    pub fn sample2d(&self, x: f32, y: f32) -> f32 {
        let xf = x * self.frequency;
        let yf = y * self.frequency;
        match self.fractal {
            None => self.single2d(xf, yf),
            Some(cfg) => self.fbm2d(xf, yf, cfg),
        }
    }

    fn single2d(&self, x: f32, y: f32) -> f32 {
        match self.noise_type {
            NoiseType::Perlin => perlin2d(x, y, &self.p),
            NoiseType::Value => value2d(x, y, &self.p),
        }
    }

    fn fbm2d(&self, mut x: f32, mut y: f32, cfg: FractalCfg) -> f32 {
        let mut amp = 0.5; // стартовая амплитуда
        let mut sum = 0.0;
        let mut norm = 0.0;
        for _ in 0..cfg.octaves.max(1) {
            match self.noise_type {
                NoiseType::Perlin => sum += perlin2d(x, y, &self.p) * amp,
                NoiseType::Value => sum += value2d(x, y, &self.p) * amp,
            }
            norm += amp;
            x *= cfg.lacunarity;
            y *= cfg.lacunarity;
            amp *= cfg.gain;
        }
        if norm > 0.0 { sum / norm } else { sum }
    }
}

// =================
// PRNG: xorshift64*
// =================
#[derive(Clone, Copy, Debug)]
struct XorShift64Star(u64);
impl XorShift64Star {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }
}

// =================
// Вспомогательные функции
// =================
#[inline]
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

// Для Perlin градиент определяется по хэшу
#[inline]
fn grad2(hash: u8, x: f32, y: f32) -> f32 {
    // 8 направлений
    match hash & 7 {
        0 => x + y,
        1 => x - y,
        2 => -x + y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

// =================
// Value noise (2D)
// =================
fn value2d(x: f32, y: f32, p: &[u8; 512]) -> f32 {
    // Грид ячейки
    let xi0 = x.floor() as i32 & 255;
    let yi0 = y.floor() as i32 & 255;
    let xi1 = (xi0 + 1) & 255;
    let yi1 = (yi0 + 1) & 255;

    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    // Псевдо-значения в вершинах
    let aa = p[(p[xi0 as usize] as usize + yi0 as usize) & 255] as f32 / 255.0;
    let ab = p[(p[xi0 as usize] as usize + yi1 as usize) & 255] as f32 / 255.0;
    let ba = p[(p[xi1 as usize] as usize + yi0 as usize) & 255] as f32 / 255.0;
    let bb = p[(p[xi1 as usize] as usize + yi1 as usize) & 255] as f32 / 255.0;

    let x1 = lerp(aa, ba, u);
    let x2 = lerp(ab, bb, u);
    let v = lerp(x1, x2, v);
    // Растянем к [-1, 1]
    v * 2.0 - 1.0
}

// =================
// Perlin noise (2D)
// =================
fn perlin2d(x: f32, y: f32, p: &[u8; 512]) -> f32 {
    // Нижняя целочисленная координата
    let xi0 = x.floor() as i32 & 255;
    let yi0 = y.floor() as i32 & 255;
    let xi1 = (xi0 + 1) & 255;
    let yi1 = (yi0 + 1) & 255;

    // Локальные координаты внутри клетки
    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    // Хэши для 4 углов
    let aa = p[(p[xi0 as usize] as usize + yi0 as usize) & 255];
    let ab = p[(p[xi0 as usize] as usize + yi1 as usize) & 255];
    let ba = p[(p[xi1 as usize] as usize + yi0 as usize) & 255];
    let bb = p[(p[xi1 as usize] as usize + yi1 as usize) & 255];

    // Скалярные произведения с градиентами в углах
    let x1 = lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u);
    // Интерполяция по y
    let val = lerp(x1, x2, v);
    // Нормализация в ~[-1, 1]
    val * 0.7071 // тонкая коррекция амплитуды
}

// =================
// Тесты и пример
// =================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_runs() {
        let n = FastNoise::new(123)
            .with_frequency(0.02)
            .with_noise_type(NoiseType::Perlin);
        let v = n.sample2d(10.5, -7.25);
        assert!(v >= -1.1 && v <= 1.1);
    }

    #[test]
    fn fbm_changes_range() {
        let base = FastNoise::new(42).with_frequency(0.03);
        let single = base.sample2d(1.0, 2.0);
        let fbm = base
            .clone()
            .with_fractal(FractalCfg {
                octaves: 5,
                lacunarity: 2.0,
                gain: 0.5,
            })
            .sample2d(1.0, 2.0);
        // значения просто разные (не NaN)
        assert!(single.is_finite() && fbm.is_finite() && (single - fbm).abs() > 1e-6);
    }
}

/* =================
Пример использования в бинарнике:

use fastnoise::{FastNoise, NoiseType, FractalCfg};

fn main() {
    let noise = FastNoise::new(2025)
        .with_frequency(0.01)
        .with_noise_type(NoiseType::Perlin)
        .with_fractal(FractalCfg { octaves: 6, lacunarity: 2.0, gain: 0.5 });

    // семплируем сетку 256x256 и выводим как PGM
    let w = 256usize; let h = 256usize;
    let mut buf = Vec::with_capacity(w*h);
    for y in 0..h {
        for x in 0..w {
            let v = noise.sample2d(x as f32, y as f32); // [-1,1]
            let g = (((v + 1.0) * 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            buf.push(g);
        }
    }
    // далее можно сохранить через image/pgm/crate по вкусу
}

=================
Расширения:
- Добавить 3D: по аналогии с 2D, но с 8 вершинами куба и grad3().
- Добавить Ridged/Hybrid мультифракталы.
- Domain warp (перекос координат второй шумовой функцией).
- SIMD/parallel (rayon + packed_simd/portable_simd).
- Таблицы градиентов с 256 направлений для более мягких структур.
*/
