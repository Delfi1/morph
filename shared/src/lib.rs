//! Todo: Init rune server
//!
//! Morph: voxel engine with server-side mesher
//! Mesher on Rune

use std::{collections::*, sync::*};

pub use bevy_math as math;
pub use bevy_tasks as tasks;

use math::*;
use rune::Module;
use tasks::*;

pub mod chunk;
pub mod mesh;

use chunk::*;
use mesh::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ScriptType {
    Mesher,
    Generator,
    // Ticker unit, object with ticks
    Ticker(u32),
}

static VALUE: OnceLock<Core> = OnceLock::new();

pub struct Core {
    chunks: Mutex<HashMap<IVec3, Chunk>>,
    meshes: Mutex<HashMap<IVec3, Mesh>>,

    /// Scripts units
    units: Mutex<HashMap<ScriptType, Arc<rune::Unit>>>,

    // Thread tasks:
    /// Gen Tasks
    gtasks: Mutex<HashMap<IVec3, Task<Chunk>>>,
    /// Mesh tasks
    mtasks: Mutex<HashMap<IVec3, Task<Mesh>>>,
}

/// Init main shared components and core data
pub fn init() {
    tasks::ComputeTaskPool::get_or_init(|| TaskPool::new());

    let _ = VALUE.set(Core {
        chunks: Mutex::new(HashMap::new()),
        meshes: Mutex::new(HashMap::new()),

        units: Mutex::new(HashMap::new()),

        gtasks: Mutex::new(HashMap::new()),
        mtasks: Mutex::new(HashMap::new()),
    });
}

pub fn get_chunk(pos: IVec3) -> Option<Chunk> {
    let value = VALUE.get().unwrap();
    let guard = value.chunks.lock().unwrap();

    guard.get(&pos).cloned()
}

pub fn add_chunk(pos: IVec3, chunk: Chunk) {
    let value = VALUE.get().unwrap();
    let mut guard = value.chunks.lock().unwrap();

    guard.insert(pos, chunk);
}

pub fn add_ticker(id: u32, unit: Arc<rune::Unit>) {
    let value = VALUE.get().unwrap();
    let mut guard = value.units.lock().unwrap();

    guard.insert(ScriptType::Ticker(id), unit);
}

pub fn proceed_worldgen(pos: IVec3) {
    let value = VALUE.get().unwrap();
    let guard = value.units.lock().unwrap();

    let Some(generator) = guard.get(&ScriptType::Generator) else {
        log::error!("Generator script is not initialized");
        return;
    };
}

pub fn proceed_mesher(pos: IVec3) {
    let value = VALUE.get().unwrap();
    let guard = value.units.lock().unwrap();

    let Some(mesher) = guard.get(&ScriptType::Mesher) else {
        log::error!("Mesher script is not initialized");
        return;
    };
}

/// IVec3's representation in Rune
#[derive(rune::Any, Clone)]
pub struct RnIVec3(IVec3);

impl RnIVec3 {
    pub fn new(position: IVec3) -> Self {
        Self(position)
    }
}

#[rune::function]
pub fn ivec3(x: i32, y: i32, z: i32) -> RnIVec3 {
    RnIVec3(IVec3::new(x, y, z))
}

#[rune::function]
pub fn get_block(chunk: &RnIVec3, block: &RnIVec3) -> u16 {
    let chunk = get_chunk(chunk.0).unwrap();
    let data = chunk.read();

    data.get_block(RawChunk::block_index(block.0))
}

/// get chunk refs block
#[rune::function(instance)]
pub fn refs_block(refs: &ChunksRefs, pos: RnIVec3) -> u16 {
    refs.get_block(pos.0)
}

/// Setup module
pub fn module(context: &mut rune::Context) -> rune::support::Result<()> {
    let mut m = Module::new();

    // Helpful functions
    m.function_meta(ivec3);

    // Chunks functions
    m.function_meta(get_block);
    m.function_meta(refs_block);

    Ok(())
}
