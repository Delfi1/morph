use std::{sync::*, collections::*};

#[derive(rune::Any, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    #[rune(constructor)]
    Left,
    #[rune(constructor)]
    Right,
    #[rune(constructor)]
    Down,
    #[rune(constructor)]
    Up,
    #[rune(constructor)]
    Back,
    #[rune(constructor)]
    Forward,
}

#[derive(Debug, Clone, Copy, rune::Any)]
pub enum ModelType {
    #[rune(constructor)]
    Full,
    #[rune(constructor)]
    Slab,
    #[rune(constructor)]
    Stair,
    //Custom(#[rune(get)] u32)
}

#[derive(Debug, rune::Any, Clone)]
pub struct Model {
    model: ModelType,

    /// Texture's path
    pub texture: String,
}

static VALUE: OnceLock<BlocksHandler> = OnceLock::new();

#[derive(Debug)]
struct BlocksHandler {
    blocks: RwLock<Vec<BlockType>>,
    names: RwLock<HashMap<String, u32>>,
}

pub(crate) fn init_blocks() {
    let blocks = RwLock::new(Vec::new());
    let names = RwLock::new(HashMap::new());

    VALUE.set(BlocksHandler { blocks, names }).unwrap();
}

#[derive(Debug, rune::Any, Clone)]
pub struct BlockType {
    pub model: Option<Model>,
    // todo: light, collisions etc
}

#[rune::function]
pub fn new_model(ty: ModelType, texture: String) -> Model {
    Model { model: ty, texture }
}

#[rune::function]
pub fn clear_blocks() {
    let handler = VALUE.get().unwrap();
    let mut guard = handler.names.write().unwrap();
    guard.clear();

    let mut guard = handler.blocks.write().unwrap();
    guard.clear();
}

/// Add block to a handler
#[rune::function]
pub fn add_block(name: String, model: Option<Model>) {
    let handler = VALUE.get().unwrap();

    let mut names = handler.names.write().unwrap();
    if let Some(id) = names.get(&name) { 
        let mut blocks = handler.blocks.write().unwrap();
        blocks[*id as usize] = BlockType { model };
        
        return;
    }

    let id = names.len() as u32;
    names.insert(name, id);

    let mut blocks = handler.blocks.write().unwrap();
    blocks.push(BlockType { model });
}

/// Get block data by type
#[rune::function]
pub fn block_type(id: u32) -> Option<BlockType> {
    let handler = VALUE.get().unwrap();
    let guard = handler.blocks.read().unwrap();

    guard.get(id as usize).cloned()
}

#[rune::function]
pub fn model_type(block: &BlockType) -> Option<ModelType> {
    block.model.as_ref().and_then(|m| Some(m.model))
}
