use spacetimedb::{table, SpacetimeType};

#[derive(SpacetimeType)]
#[derive(serde::Serialize, serde::Deserialize)]
// Model type with texture file name
pub enum ModelType {
    Cube(String),
    Stair(String),
    Slab(String),
    Custom(String),
    Empty
}

// Block type table
#[table(name = block)]
pub struct Block {
    #[primary_key]
    pub id: u16,
    // block name
    #[unique]
    pub name: String,
    // Texture path and model
    pub model: ModelType,
    // light?
    // collision? todo
}
