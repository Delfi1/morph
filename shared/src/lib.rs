//! Todo: Init rune server
//!
//! Morph: voxel engine with server-side mesher
//! Mesher on Rune

use std::{collections::*, io::Write, sync::*};

// Re-exports
pub use fastnoise_lite as noise;
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
    entry: Option<String>,

    #[rune(set)]
    /// Count of one-time operations, one by default
    threading: u32,
}

impl Default for ScriptMeta {
    fn default() -> Self {
        Self { threading: 1, entry: None }
    }
}

#[rune::function]
/// Create new script metadata
pub fn meta(entry: String, threading: u32) -> ScriptMeta {
    ScriptMeta { entry: Some(entry), threading }
}

// TODO: Script return data 
//pub struct ScriptResult {}

#[derive(Debug)]
/// Script compiled and meta data
struct Script {
    unit: Arc<rune::Unit>,
    sources: Arc<rune::Sources>,
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

    let mut guard = scripts.tasks.lock().unwrap();
    guard.clear();
}

/// Insert new script by type
pub fn insert_script(path: String, raw: impl AsRef<str>) -> rune::support::Result<()> {
    let scripts = SCRIPTS.get().unwrap();

    // Remove script if exists
    remove_script(&path);

    let mut sources = rune::Sources::new();
    sources.insert(rune::Source::memory(raw)?)?;

    let result = rune::prepare(&mut sources)
        .with_context(&scripts.context)
        .build();

    let unit = match result {
        Ok(unit) => Arc::new(unit),
        Err(e) => {
            log::error!("Script build error: {}", e);

            // Compile error was processed
            return Ok(());
        }        
    };

    let sources = Arc::new(sources);
    let mut vm = rune::Vm::new(scripts.runtime.clone(), unit.clone());

    // Init scripts and get metadata
    let result = vm.call(["init"], ())
        .and_then(|v| Ok(rune::from_value::<ScriptMeta>(v)?));

    let meta = match result {
        Ok(meta) => meta,
        Err(e) => { 
            log::warn!("Script init error: {}", e);
            ScriptMeta::default()
        }
    };
        
    // Don't add script if it don't have entry point
    if meta.entry.is_none() { return Ok(()); }

    // Create tasks variable
    let mut tasks = scripts.tasks.lock().unwrap();
    tasks.insert(path.clone(), Vec::new());

    let mut guard = scripts.values.write().unwrap();
    guard.insert(path, Script { unit, meta, sources });

    Ok(())
}

/// Insert new script by type
pub fn remove_script(path: &String) {
    let scripts = SCRIPTS.get().unwrap();

    let mut guard = scripts.values.write().unwrap();

    // Remove tasks if exists
    if let Some(_) = guard.remove(path) {
        let mut tasks = scripts.tasks.lock().unwrap();
        tasks.remove(path);
    }
}

pub async fn run_script(
    runtime: Arc<rune::runtime::RuntimeContext>, 
    entry: String,
    unit: Arc<rune::Unit>, 
    sources: Arc<rune::Sources>
) {
    let mut vm = rune::Vm::new(runtime.clone(), unit.clone());
    
    let mut stream = rune::termcolor::BufferedStandardStream::stdout(Default::default());
    let mut buffer = String::new();

    let mut diag = rune::Diagnostics::new();
    if let Err(_) = vm.call_with_diagnostics([entry.as_str()], (), Some(&mut diag)) {
        diag.emit(&mut stream, &sources).unwrap();
        // Safety: output stream is always UTF-8
        unsafe { stream.write_all(buffer.as_bytes_mut()).unwrap(); }

        log::error!("Script execute error diagnostics: {}", buffer);
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

        let entry = script.meta.entry.clone().unwrap();
        let unit = script.unit.clone();
        let sources = script.sources.clone();

        // Spawn task and insert
        new.push(taskpool.spawn(run_script(runtime.clone(), entry, unit, sources)));

        tasks_guard.insert(path.clone(), new);
    }

    Ok(())
}

static CORE: OnceLock<Core> = OnceLock::new();

pub struct Core {
    chunks: Mutex<HashMap<IVec3, Chunk>>,
    meshes: Mutex<HashMap<IVec3, Mesh>>,

    gen_queue: Mutex<VecDeque<IVec3>>,
    meshes_queue: Mutex<VecDeque<IVec3>>,

    //noise: Mutex<noise::FastNoiseLite>
}

pub fn is_initalized() -> bool {
    CORE.get().is_some()
}

/// Init main shared components and core data
pub fn init() {
    AsyncComputeTaskPool::get_or_init(|| TaskPool::new());

    init_blocks();
    init_scripts().expect("Scripts initialization error");

    if CORE.set(Core {
        chunks: Mutex::new(HashMap::new()),
        meshes: Mutex::new(HashMap::new()),

        gen_queue: Mutex::new(VecDeque::new()),
        meshes_queue: Mutex::new(VecDeque::new()),
        
        //noise: Mutex::new(noise::FastNoiseLite::new())
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

#[rune::macro_]
fn f(
    cx: &mut rune::macros::MacroContext<'_, '_, '_>, 
    stream: &rune::macros::TokenStream
) -> rune::compile::Result<rune::macros::TokenStream>  {
    let mut parser = rune::parse::Parser::from_token_stream(stream, cx.input_span());
    let mut output = rune::macros::quote!("");

    while !parser.is_eof()? {
        let ident = parser.parse_all::<rune::ast::Ident>()?;
        let ident = rune::alloc::borrow::TryToOwned::try_to_owned(cx.resolve(ident)?)?;
        let value = cx.lit(ident)?;

        output = rune::macros::quote!(#output + #value);
    }

    parser.eof()?;
    Ok(rune::macros::quote!(#output).into_token_stream(cx)?)
}

/*
#[rune::macro_]
fn ident_to_string(cx: &mut MacroContext<'_, '_, '_>, stream: &TokenStream) -> compile::Result<TokenStream> {
    let mut p = Parser::from_token_stream(stream, cx.input_span());
    let ident = p.parse_all::<ast::Ident>()?;
    let ident = cx.resolve(ident)?.try_to_owned()?;
    let string = cx.lit(&ident)?;
    Ok(quote!(#string).into_token_stream(cx)?)
}
*/

#[rune::function]
fn debug(value: rune::Value) {
    match value.borrow_string_ref() {
        Ok(output) => log::debug!("{}", output.to_string()),
        Err(_) => log::debug!("{:?}", value)
    }
}

// todo: Noise for worldgen

#[rune::function]
fn get_chunk(pos: RnIVec3) -> Option<Chunk> {
    _get_chunk(pos.0)
}

#[rune::function]
fn add_chunk(chunk: Chunk, pos: RnIVec3) {
    let core = CORE.get().unwrap();
    let mut guard = core.chunks.lock().unwrap();

    guard.insert(pos.0, chunk);
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
/// Return mesh position back to the list
fn return_mesh(pos: RnIVec3) {
    let core = CORE.get().unwrap();
    let mut queue = core.meshes_queue.lock().unwrap();
    
    queue.push_back(pos.0)
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
    m.ty::<ModelType>()?;
    m.ty::<Model>()?;
    m.ty::<BlockType>()?;
    m.ty::<Chunk>()?;
    m.ty::<ChunksRefs>()?;
    m.ty::<Direction>()?;
    m.ty::<Mesh>()?;

    // Helpful functions
    m.macro_meta(f)?;

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

    // Blocks functions
    m.function_meta(new_model)?;
    m.function_meta(clear_blocks)?;
    m.function_meta(add_block)?;
    m.function_meta(block_type)?;
    m.function_meta(block_id)?;
    m.function_meta(model_type)?;
    m.function_meta(position_index)?;

    // Meshes
    m.function_meta(new_mesh)?;
    m.function_meta(add_mesh)?;

    // Requests to a Core
    m.function_meta(request_gen)?;
    m.function_meta(request_mesh)?;
    m.function_meta(return_mesh)?;

    context.install(m)?;
    Ok(())
}
