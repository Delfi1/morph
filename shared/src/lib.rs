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

/// Create scripts context and install main Morph module
fn init_scripts() -> rune::support::Result<()> {
    let mut context = rune::Context::with_default_modules()?;
    module(&mut context)?;

    let runtime = Arc::new(context.runtime()?);
    let units = RwLock::new(HashMap::new());

    let _ = SCRIPTS.set(Scripts { context, runtime, units });
    Ok(())
}

/// Insert new script by type
pub fn insert_script(raw: String, script_type: ScriptType) -> rune::support::Result<()> {
    let scripts = SCRIPTS.get().unwrap();

    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::memory(raw)?)?;

    let unit = rune::prepare(&mut sources)
        .with_context(&scripts.context)
        .build()?;

    let unit = Arc::new(unit);

    let mut guard = scripts.units.write().unwrap();
    guard.insert(script_type, unit);

    Ok(())
}

/// Init Rune command
pub async fn build_mesh_task(unit: Arc<rune::Unit>, pos: IVec3) -> Option<Mesh> {
    let runtime = SCRIPTS.get().unwrap().runtime.clone();
    let mut vm = rune::Vm::new(runtime, unit);

    let output = vm.call(["mesher"], (RnIVec3(pos), )).ok()?;
    let result: Option<Mesh> = rune::from_value(output).ok()?;
    
    result
}

pub async fn generate_chunk_task(unit: Arc<rune::Unit>, pos: IVec3) -> Chunk {
    let runtime = SCRIPTS.get().unwrap().runtime.clone();
    let mut vm = rune::Vm::new(runtime, unit);

    let output = vm.call(["mesher"], (RnIVec3(pos), )).unwrap();
    let result: Chunk = rune::from_value(output).unwrap();
    
    result
}

/// Run worldgen script as `bevy` Task
pub fn proceed_worldgen(pos: IVec3) {
    let value = VALUE.get().unwrap();
    let scripts = SCRIPTS.get().unwrap();
    
    let guard = scripts.units.read().unwrap();
    let Some(generator) = guard.get(&ScriptType::Generator).cloned() else {
        log::error!("Generator script is not initialized");
        return;
    };

    // Create tasks hashmap guard
    let mut guard = value.gtasks.lock().unwrap();
    let taskpool = tasks::AsyncComputeTaskPool::get();
    let task = taskpool.spawn(generate_chunk_task(generator, pos));
    
    guard.insert(pos, task);
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
    tasks::AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

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

/// Proceed Compute task pool tasks
pub fn proceed_tasks() {
    let taskpool = tasks::AsyncComputeTaskPool::get();

    taskpool.with_local_executor(|ex| ex.try_tick());
}

#[rune::function]
pub fn debug(value: rune::Value) {
    log::debug!("{:?}", value);
}

/// Setup module
pub fn module(context: &mut rune::Context) -> rune::support::Result<()> {
    let mut m = rune::Module::new();

    // Main types
    m.type_meta::<ChunksRefs>()?;
    m.type_meta::<Direction>()?;
    m.type_meta::<Mesh>()?;

    // Helpful functions
    m.function_meta(chunk::ivec3)?;
    m.function_meta(debug)?;

    // Chunks functions
    m.function_meta(get_block)?;
    m.function_meta(set_block)?;
    m.function_meta(get_refs)?;
    m.function_meta(refs_block)?;

    // Meshes
    m.function_meta(new_mesh)?;
    m.function_meta(push_vertex)?;
    m.function_meta(finish_mesh)?;

    context.install(m)?;
    Ok(())
}
