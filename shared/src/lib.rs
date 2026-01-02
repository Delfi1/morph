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

/// Max mesh tasks at time
pub const MESH_TASKS: u32 = 32;
pub const GEN_TASKS: u32 = 64;

static SCRIPTS: OnceLock<Scripts> = OnceLock::new();

#[derive(Debug)]
struct Scripts {
    context: rune::Context,
    runtime: Arc<rune::runtime::RuntimeContext>,

    /// Scripts units
    units: RwLock<HashMap<String, Arc<rune::Unit>>>,
}

/// Create scripts context and install main Morph module
fn init_scripts() -> rune::support::Result<()> {
    let mut context = rune::Context::with_default_modules()?;
    module(&mut context)?;

    let runtime = Arc::new(context.runtime()?);
    let units = RwLock::new(HashMap::new());

    if SCRIPTS.set(Scripts { context, runtime, units }).is_err() {
        log::error!("Already initialized");
    }
    Ok(())
}

pub fn clear_scripts() {
    let scripts = SCRIPTS.get().unwrap();
    let mut guard = scripts.units.write().unwrap();
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

    let mut guard = scripts.units.write().unwrap();
    guard.insert(path, unit);

    Ok(())
}

/// Insert new script by type
pub fn remove_script(path: String) {
    let scripts = SCRIPTS.get().unwrap();

    let mut guard = scripts.units.write().unwrap();
    guard.remove(&path);
}

pub async fn run_tick(runtime: Arc<rune::runtime::RuntimeContext>, unit: Arc<rune::Unit>) {
    let mut vm = rune::Vm::new(runtime.clone(), unit.clone());

    if let Err(e) = vm.call(["tick"], ()) {
        log::error!("Script execute error: {}", e);
    }
}

/// Call all tickers scrits
pub fn tick_scripts() -> rune::support::Result<()> {
    let scripts = SCRIPTS.get().unwrap();
    let runtime = scripts.runtime.clone();
    let taskpool = AsyncComputeTaskPool::get();

    let guard = scripts.units.read().unwrap();
    for (_, unit) in guard.iter() {
        // Spawn detached tasks
        taskpool.spawn(run_tick(runtime.clone(), unit.clone())).detach();
    }

    Ok(())
}

static CORE: OnceLock<Core> = OnceLock::new();

#[derive(Debug)]
pub struct Core {
    chunks: Mutex<HashMap<IVec3, Chunk>>,
    meshes: Mutex<HashMap<IVec3, Mesh>>,

    gen_tasks: atomic::AtomicU32,
    gen_queue: Mutex<VecDeque<IVec3>>,
    meshes_tasks: atomic::AtomicU32,
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

        gen_tasks: 0.into(),
        gen_queue: Mutex::new(VecDeque::new()),
        meshes_tasks: 0.into(),
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
    let tasks = core.gen_tasks.load(atomic::Ordering::Relaxed);
    
    if tasks >= GEN_TASKS { return None; }

    match queue.pop_back() {
        None => None,
        Some(p) => {
            core.gen_tasks.fetch_add(1, atomic::Ordering::Relaxed);
            Some(RnIVec3(p))
        }
    }
}

#[rune::function]
/// Request chunk position from mesher queue
fn request_mesh() -> Option<RnIVec3> {
    let core = CORE.get().unwrap();
    let mut queue = core.meshes_queue.lock().unwrap();
    let tasks = core.meshes_tasks.load(atomic::Ordering::Relaxed);
    
    if tasks >= MESH_TASKS { return None; }

    match queue.pop_back() {
        None => None,
        Some(p) => {
            core.meshes_tasks.fetch_add(1, atomic::Ordering::Relaxed);
            Some(RnIVec3(p))
        }
    }
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
    m.function_meta(request_gen)?;
    m.function_meta(request_mesh)?;

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

    context.install(m)?;
    Ok(())
}
