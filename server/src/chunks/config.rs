use crate::chunks::fastnoise::{FastNoise, FractalCfg, NoiseType};
use serde::Deserialize;
use spacetimedb::ReducerContext;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct WorldConfig {
    pub chunk_range: i32,
    pub chunk_height_range: i32,
    pub chunk_bottom_range: i32,
    pub world_height_render: i32,
    pub world_bottom_render: i32,
    pub range_render: i32,
}

const DEFAULT_GEN_JSON: &str = r#"
{
  "chunk_range": 4,
  "chunk_height_range": 4,
  "chunk_bottom_range": -2,
  "world_height_render": 5,
  "world_bottom_render": -3,
  "range_render": 6
}
"#;

// Единичный слой шума из файла
#[derive(Debug, Deserialize, Clone)]
pub struct NoiseLayerConfig {
    pub name: String,
    pub seed: u64,
    pub frequency: f32,
    pub octaves: u8,
    pub lacunarity: f32,
    pub gain: f32,
    pub noise_type: String,
    pub base_level_blocks: i32,
    pub amplitude_blocks: i32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NoiseBankConfig {
    pub noises: Vec<NoiseLayerConfig>,
}

const DEFAULT_NOISES_JSON: &str = r#"
{

    {
      "name": "base",
      "seed": 12345,
      "frequency": 0.02,
      "octaves": 5,
      "lacunarity": 0.9,
      "gain": 0.5,
      "noise_type": "Perlin",
      "base_level_blocks": 4,
      "amplitude_blocks": 14
    },
    {
      "name": "mountain_mask",
      "seed": 12345,
      "frequency": 0.005,
      "octaves": 2,
      "lacunarity": 2.0,
      "gain": 0.5,
      "noise_type": "Perlin",
      "base_level_blocks": 0,
      "amplitude_blocks": 1
    },
    {
      "name": "mountain_height",
      "seed": 12345,
      "frequency": 0.015,
      "octaves": 7,
      "lacunarity": 2.0,
      "gain": 0.5,
      "noise_type": "Perlin",
      "base_level_blocks": 64,
      "amplitude_blocks": 196
    },
}
"#;

// Подготовленный слой шума для генерации
#[derive(Clone)]
pub struct NoiseLayer {
    pub name: String,
    pub noise: FastNoise,
    pub base_level_blocks: i32,
    pub amplitude_blocks: i32,
}

pub type NoiseBank = HashMap<String, NoiseLayer>;

impl WorldConfig {
    pub fn load(ctx: &ReducerContext) -> Self {
        crate::assets::get_json_asset::<WorldConfig>(ctx, "gen.json", DEFAULT_GEN_JSON)
    }
}

// Загрузка сырого конфига
pub fn load_noise_bank_config(ctx: &ReducerContext) -> NoiseBankConfig {
    crate::assets::get_json_asset::<NoiseBankConfig>(ctx, "noises.json", DEFAULT_NOISES_JSON)
}

// Построение банка шумов (FastNoise + параметры высоты)
pub fn build_noise_bank(ctx: &ReducerContext) -> NoiseBank {
    let cfg = load_noise_bank_config(ctx);

    let mut bank: NoiseBank = HashMap::new();
    for layer in cfg.noises.into_iter() {
        let ty = match layer.noise_type.as_str() {
            "Value" => NoiseType::Value,
            "Perlin" => NoiseType::Perlin,
            _ => NoiseType::Perlin,
        };

        let noise = FastNoise::new(layer.seed)
            .with_frequency(layer.frequency)
            .with_noise_type(ty)
            .with_fractal(FractalCfg {
                octaves: layer.octaves,
                lacunarity: layer.lacunarity,
                gain: layer.gain,
            });

        bank.insert(
            layer.name.clone(),
            NoiseLayer {
                name: layer.name,
                noise,
                base_level_blocks: layer.base_level_blocks,
                amplitude_blocks: layer.amplitude_blocks,
            },
        );
    }

    bank
}

// Удобный доступ к слою: если нет по имени — None
pub fn get_noise_layer<'a>(bank: &'a NoiseBank, name: &str) -> Option<&'a NoiseLayer> {
    bank.get(name)
}
