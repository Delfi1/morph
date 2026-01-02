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
pub mod assets;
pub mod chunk;
pub mod mesh;

use math::*;
use tasks::*;

use chunk::*;
use mesh::*;

/// Script metadata 
#[derive(Debug, rune::Any)]
pub struct ScriptMeta {
    // Entry point (function)
    #[rune(set)]
    entry: String,

    #[rune(set)]
    /// Count of one-time operations, one by default
    threading: u32,
}

impl Default for ScriptMeta {
    fn default() -> Self {
        Self { threading: 1, entry: "tick".into() }
    }
}

#[rune::function]
/// Create new script metadata
pub fn meta(entry: String, threading: u32) -> ScriptMeta {
    ScriptMeta { entry, threading }
}

// TODO: Script return data 
//pub struct ScriptResult {}

#[derive(Debug)]
/// Script compiled and meta data
struct Script {
    unit: Arc<rune::Unit>,
    meta: ScriptMeta
}

static SCRIPTS: OnceLock<Scripts> = OnceLock::new();

#[derive(Debug)]
struct Scripts {
    context: rune::Context,
    runtime: Arc<rune::runtime::RuntimeContext>,

    // TODO: create rune function return result Value (optional)
    
    /// One-time tasks
    tasks: Mutex<HashMap<String, Vec<Task<()>>>>,

    values: RwLock<HashMap<String, Script>>,
}

/// Create scripts context and install main Morph module
fn init_scripts() -> rune::support::Result<()> {
    let mut context = rune::Context::with_default_modules()?;
    module(&mut context)?;

    let runtime = Arc::new(context.runtime()?);
    let tasks = Mutex::new(HashMap::new());

    let values = RwLock::new(HashMap::new());

    if SCRIPTS.set(Scripts { context, runtime, tasks, values }).is_err() {
        log::error!("Already initialized");
    }
    Ok(())
}

pub fn clear_scripts() {
    let scripts = SCRIPTS.get().unwrap();
    let mut guard = scripts.values.write().unwrap();
    guard.clear();
}

/// Insert new script by type
pub fn insert_script(path: String, raw: impl AsRef<str>) -> rune::support::Result<()> {
    let scripts = SCRIPTS.get().unwrap();

    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::memory(raw)?)?;

    let unit = rune::prepare(&mut sources)
        .with_context(&scripts.context)
        .build()?;

    let unit = Arc::new(unit);
    let mut vm = rune::Vm::new(scripts.runtime.clone(), unit.clone());

    // Init scripts and get metadata
    let meta = vm.call(["init"], ())
        .and_then(|v| Ok(rune::from_value::<ScriptMeta>(v)?))
        .unwrap_or(ScriptMeta::default());

    // Create tasks variable
    let mut tasks = scripts.tasks.lock().unwrap();
    tasks.insert(path.clone(), Vec::new());

    let mut guard = scripts.values.write().unwrap();
    guard.insert(path, Script { unit, meta });

    Ok(())
}

/// Insert new script by type
pub fn remove_script(path: String) {
    let scripts = SCRIPTS.get().unwrap();

    let mut guard = scripts.values.write().unwrap();

    // Remove tasks if exists
    if let Some(_) = guard.remove(&path) {
        let mut tasks = scripts.tasks.lock().unwrap();
        tasks.remove(&path);
    }
}

pub async fn run_script(runtime: Arc<rune::runtime::RuntimeContext>, entry: String, unit: Arc<rune::Unit>) {
    let mut vm = rune::Vm::new(runtime.clone(), unit.clone());

    if let Err(e) = vm.call([entry.as_str()], ()) {
        log::error!("Script execute error: {}", e);
    }
}

/// Call all tickers scrits
pub fn tick_scripts() -> rune::support::Result<()> {
    let scripts = SCRIPTS.get().unwrap();
    let runtime = scripts.runtime.clone();
    let taskpool = AsyncComputeTaskPool::get();

    let guard = scripts.values.read().unwrap();
    let mut tasks_guard = scripts.tasks.lock().unwrap();

    for (path, script) in guard.iter() {
        let tasks = tasks_guard.remove(path).unwrap();
        let count = script.meta.threading as usize;

        let mut new = Vec::with_capacity(count as usize);
        for task in tasks {
            if !task.is_finished() {
                new.push(task);
            }

            // todo: else block on task 
        }

        // Can task be spawned?
        if new.len() >= count { 
            tasks_guard.insert(path.clone(), new);
            continue; 
        }

        let entry = script.meta.entry.clone();
        let unit = script.unit.clone();

        // Spawn task and insert
        new.push(taskpool.spawn(run_script(runtime.clone(), entry, unit.clone())));

        tasks_guard.insert(path.clone(), new);
    }

    Ok(())
}

static CORE: OnceLock<Core> = OnceLock::new();

#[derive(Debug)]
pub struct Core {
    chunks: Mutex<HashMap<IVec3, Chunk>>,
    meshes: Mutex<HashMap<IVec3, Mesh>>,

    gen_queue: Mutex<VecDeque<IVec3>>,
    meshes_queue: Mutex<VecDeque<IVec3>>,
}

pub fn is_initalized() -> bool {
    CORE.get().is_some()
}

/// Init main shared components and core data
pub fn init() {
    AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

    init_scripts().expect("Scripts initialization error");

    if CORE.set(Core {
        chunks: Mutex::new(HashMap::new()),
        meshes: Mutex::new(HashMap::new()),

        gen_queue: Mutex::new(VecDeque::new()),
        meshes_queue: Mutex::new(VecDeque::new()),
    }).is_err() {
        log::error!("Already initialized");
    }
}

impl Into<RnIVec3> for IVec3 {
    fn into(self) -> RnIVec3 { RnIVec3(self) }
}

pub fn add_chunk_raw(pos: IVec3) -> Option<Chunk> {
    let core = CORE.get().unwrap();
    let guard = core.chunks.lock().unwrap();

    guard.get(&pos).cloned()
}

/// Get chunk manually
pub fn _get_chunk(pos: IVec3) -> Option<Chunk> {
    let core = CORE.get().unwrap();
    let guard = core.chunks.lock().unwrap();

    guard.get(&pos).cloned()
}

#[rune::function]
fn get_chunk(pos: RnIVec3) -> Option<Chunk> {
    _get_chunk(pos.0)
}

#[rune::function]
fn add_chunk(pos: RnIVec3, chunk: Chunk) {
    let core = CORE.get().unwrap();
    let mut guard = core.chunks.lock().unwrap();

    guard.insert(pos.0, chunk);
}

#[rune::function]
fn debug(value: rune::Value) {
    log::debug!("{:?}", value);
}

#[rune::function]
/// Request chunk position from generator queue
fn request_gen() -> Option<RnIVec3> {
    let core = CORE.get().unwrap();
    let mut queue = core.gen_queue.lock().unwrap();
   
    queue.pop_back().and_then(|p| Some(RnIVec3(p)))
}

#[rune::function]
/// Request chunk position from mesher queue
fn request_mesh() -> Option<RnIVec3> {
    let core = CORE.get().unwrap();
    let mut queue = core.meshes_queue.lock().unwrap();
    
    queue.pop_back().and_then(|p| Some(RnIVec3(p)))
}

#[rune::function]
/// Add mesh to a core
/// TODO: add position value to intermediate buffer 
fn add_mesh(mesh: Mesh, pos: RnIVec3) {
    let core = CORE.get().unwrap();
    let mut meshes = core.meshes.lock().unwrap();

    meshes.insert(pos.0, mesh);
}

/// Setup module
pub fn module(context: &mut rune::Context) -> rune::support::Result<()> {
    let mut m = rune::Module::new();

    // Constants
    m.constant("SIZE", SIZE).build()?;
    m.constant("SIZE_P3", SIZE_P3).build()?;

    // Main types
    m.ty::<Chunk>()?;
    m.ty::<ChunksRefs>()?;
    m.ty::<Direction>()?;
    m.ty::<Mesh>()?;

    // Helpful functions
    m.function_meta(chunk::ivec3)?;
    m.function_meta(debug)?;
    m.function_meta(meta)?;

    // Chunks functions
    m.function_meta(new_chunk)?;
    m.function_meta(get_chunk)?;
    m.function_meta(add_chunk)?;

    m.function_meta(get_block)?;
    m.function_meta(set_block)?;

    m.function_meta(get_refs)?;
    m.function_meta(refs_block)?;

    // Meshes
    m.function_meta(new_mesh)?;
    m.function_meta(add_mesh)?;

    // Core requests
    m.function_meta(request_gen)?;
    m.function_meta(request_mesh)?;

    context.install(m)?;
    Ok(())
}
