//! Todo: Init rune server
//!
//! Morph: voxel engine with server-side mesher
//! Mesher on Rune

use std::{collections::*, sync::*};

// Re-exports
pub use bevy_math as math;
pub use bevy_tasks as tasks;
pub use rune;

// Exports
pub mod chunk;
pub mod mesh;

use math::*;
use tasks::*;

use chunk::*;
use mesh::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum ScriptType {
    Mesher,
    Generator,
    // Ticker unit, object with ticks
    Ticker(u32),
}

static SCRIPTS: OnceLock<Scripts> = OnceLock::new();
struct Scripts {
    context: rune::Context,
    runtime: Arc<rune::runtime::RuntimeContext>,

    /// Scripts units
    units: RwLock<HashMap<ScriptType, Arc<rune::Unit>>>,
}

fn init_scripts() -> rune::support::Result<()> {
    let context = rune::Context::new();
    let runtime = Arc::new(context.runtime()?);
    let units = RwLock::new(HashMap::new());

    let _ = SCRIPTS.set(Scripts { context, runtime, units });

    Ok(())
}

pub fn add_ticker(id: u32, unit: Arc<rune::Unit>) {
    let value = SCRIPTS.get().unwrap();
    let mut guard = value.units.write().unwrap();

    guard.insert(ScriptType::Ticker(id), unit);
}

/// Init Rune command
pub async fn build_mesh_task(unit: Arc<rune::Unit>, pos: IVec3) -> Option<Mesh> {
    let scripts = SCRIPTS.get().unwrap();
    let runtime = scripts.runtime.clone();
    let mut vm = rune::Vm::new(runtime, unit);

    let output = vm.call(["mesher"], (RnIVec3(pos), )).ok()?;
    let result: Option<Mesh> = rune::from_value(output).ok()?;
    
    result
}

/// Run worldgen script as `bevy` Task
pub fn proceed_worldgen(pos: IVec3) {
    let scripts = SCRIPTS.get().unwrap();
    let guard = scripts.units.read().unwrap();

    let Some(generator) = guard.get(&ScriptType::Generator) else {
        log::error!("Generator script is not initialized");
        return;
    };
}

/// Run mesher script as `bevy` Task
pub fn proceed_mesher(pos: IVec3) {
    let value = VALUE.get().unwrap();
    let scripts = SCRIPTS.get().unwrap();

    let guard = scripts.units.read().unwrap();
    let Some(mesher) = guard.get(&ScriptType::Mesher).cloned() else {
        log::error!("Mesher script is not initialized");
        return;
    };

    // Create tasks hashmap guard
    let mut guard = value.mtasks.lock().unwrap();
    let taskpool = tasks::AsyncComputeTaskPool::get();
    let task = taskpool.spawn(build_mesh_task(mesher, pos));
    
    guard.insert(pos, task);
}

static VALUE: OnceLock<Core> = OnceLock::new();
pub struct Core {
    chunks: Mutex<HashMap<IVec3, Chunk>>,
    meshes: Mutex<HashMap<IVec3, Mesh>>,

    // Thread tasks:
    /// Gen Tasks
    gtasks: Mutex<HashMap<IVec3, Task<Chunk>>>,
    /// Mesh tasks
    mtasks: Mutex<HashMap<IVec3, Task<Option<Mesh>>>>,
}

/// Init main shared components and core data
pub fn init() {
    tasks::ComputeTaskPool::get_or_init(|| TaskPool::new());

    init_scripts().unwrap();

    let _ = VALUE.set(Core {
        chunks: Mutex::new(HashMap::new()),
        meshes: Mutex::new(HashMap::new()),

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

/// Bevy IVec3's representation in Rune
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

#[rune::function]
/// Get chunk refs data
pub fn get_refs(pos: RnIVec3) -> Option<ChunksRefs> {
    ChunksRefs::new(pos.0)
}

/// Get chunk refs block
#[rune::function(instance)]
pub fn refs_block(refs: &ChunksRefs, pos: RnIVec3) -> u16 {
    refs.get_block(pos.0)
}

/// Setup module
pub fn module(context: &mut rune::Context) -> rune::support::Result<()> {
    let mut m = rune::Module::new();

    // Main types
    m.type_meta::<ChunksRefs>()?;
    m.type_meta::<Direction>()?;
    m.type_meta::<Mesh>()?;

    // Helpful functions
    m.function_meta(ivec3)?;

    // Chunks functions
    m.function_meta(get_block)?;
    m.function_meta(get_refs)?;
    m.function_meta(refs_block)?;

    // Meshes
    m.function_meta(new_mesh)?;
    m.function_meta(finish_mesh)?;

    context.install(m)?;
    Ok(())
}
