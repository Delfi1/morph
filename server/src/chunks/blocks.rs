use spacetimedb::{table, ReducerContext, SpacetimeType};

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

impl ModelType {
    pub fn is_meshable(&self) -> bool {
        match self {
            Self::Cube(_) => true,
            // WIP: other meshable blocks
            _ => false
        }
    }
}

// Block type table
#[table(name = block, public)]
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

pub fn is_meshable(ctx: &ReducerContext, id: u16) -> bool {
    let Some(block) = ctx.db.block().id().find(id) else {
        return false; // block is not found
    };

    block.model.is_meshable()
}