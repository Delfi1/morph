use std::sync::*;
use spacetimedb::{table, ReducerContext, Table, SpacetimeType};

#[derive(Debug)]
pub struct BlocksHandler(RwLock<Vec<Arc<Block>>>);
static VALUE: OnceLock<BlocksHandler> = OnceLock::new();

impl BlocksHandler {
    pub fn new(ctx: &ReducerContext) -> Self {
        let blocks = ctx.db.block().iter().map(|b| Arc::new(b)).collect();
        Self(RwLock::new(blocks))
    }

    pub fn init(ctx: &ReducerContext) {
        VALUE.set(Self::new(ctx)).unwrap();
    }

    pub fn get() -> &'static Self {
        VALUE.get().unwrap()
    }

    pub fn block(&self, id: u16) -> Option<Arc<Block>> {
        let access = self.0.read().unwrap();
        access.get(id as usize).cloned()
    }
}

#[derive(SpacetimeType)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
// Model type with texture file name
pub enum ModelType {
    Empty,
    Cube(String),
    Stair(String),
    Slab(String),
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

impl Block {
    pub fn is_meshable(&self) -> bool {
        self.model.is_meshable()
    }
}