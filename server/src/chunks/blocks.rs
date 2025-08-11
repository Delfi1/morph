use spacetimedb::{table, SpacetimeType};

#[derive(SpacetimeType)]
// Model type with texture file name
pub enum ModelType {
    Cube(String),
    Stair(String),
    Slab(String),
    Custom(String),
    Empty
}

#[derive(SpacetimeType)]
pub struct BlockType {
    // block type name
    pub name: String,
    // Texture path
    pub model: ModelType,
    // light?
    // collision?
}

impl BlockType {
    pub fn new(name: String, model: ModelType) -> Self {
        Self { name, model }
    }
}

// Block type table
#[table(name = block)]
pub struct Block {
    #[primary_key]
    pub id: u16,
    #[unique]
    pub block_type: BlockType
}
