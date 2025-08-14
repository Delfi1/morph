use std::sync::*;
use spacetimedb::{table, ReducerContext, Table, SpacetimeType};

#[derive(Debug)]
pub struct BlocksHandler(RwLock<Vec<Arc<Block>>>);

impl BlocksHandler {
    pub fn new(ctx: &ReducerContext) -> Self {
        Self(RwLock::new(ctx.db.block().iter().map(|b| Arc::new(b)).collect()))
    }

    pub fn get(&self, id: u16) -> Option<Arc<Block>> {
        let access = self.0.read().unwrap();
        access.get(id as usize).cloned()
    }
}

pub static BLOCKS_HANDLER: OnceLock<BlocksHandler> = OnceLock::new();

#[derive(SpacetimeType)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug)]
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

pub fn is_meshable(id: u16) -> bool {
    let Some(block) = BLOCKS_HANDLER.get().unwrap().get(id) else {
        return false; // block is not found
    };

    block.model.is_meshable()
}