use crate::chunks::fastnoise::{FastNoise, FractalCfg, NoiseType};
use serde::Deserialize;
use spacetimedb::ReducerContext;

#[derive(Debug, Deserialize, Clone)]
pub struct WorldConfig {
    pub chunk_range: i32,
    pub world_height: i32,
    pub world_bottom: i32,
    pub range_render: i32,   // <-- новое поле
}

#[derive(Debug, Deserialize, Clone)]
pub struct NoiseConfig {
    pub seed: u64,
    pub frequency: f32,
    pub octaves: u8,
    pub lacunarity: f32,
    pub gain: f32,
    pub noise_type: String,
    pub base_level_blocks: i32,
    pub amplitude_blocks: i32,
}

// --- дефолтные JSON ---
const DEFAULT_GEN_JSON: &str = r#"
{
  "chunk_range": 4,
  "world_height": 4,
  "world_bottom": -4,
  "range_render": 12
}
"#;

const DEFAULT_NOISE_JSON: &str = r#"
{
  "seed": 12345,
  "frequency": 0.02,
  "octaves": 4,
  "lacunarity": 2.0,
  "gain": 0.5,
  "noise_type": "Perlin",
  "base_level_blocks": 16,
  "amplitude_blocks": 10
}
"#;

// --- загрузка ---
impl WorldConfig {
    pub fn load(ctx: &ReducerContext) -> Self {
        crate::assets::get_json_asset::<WorldConfig>(ctx, "gen.json", DEFAULT_GEN_JSON)
    }
}

pub fn load_noise_config(ctx: &ReducerContext) -> NoiseConfig {
    crate::assets::get_json_asset::<NoiseConfig>(ctx, "noise.json", DEFAULT_NOISE_JSON)
}

pub fn get_noise(ctx: &ReducerContext) -> FastNoise {
    let cfg = load_noise_config(ctx);
    let ty = match cfg.noise_type.as_str() {
        "Value" => NoiseType::Value,
        "Perlin" => NoiseType::Perlin,
        _ => NoiseType::Perlin,
    };

    FastNoise::new(cfg.seed)
        .with_frequency(cfg.frequency)
        .with_noise_type(ty)
        .with_fractal(FractalCfg {
            octaves: cfg.octaves,
            lacunarity: cfg.lacunarity,
            gain: cfg.gain,
        })
}
