// FIXME: optimize chunks size

// todo: player access chunks - 3*3*3 chunks area
// player access meshes - 16*16*16 chunks area
// player position -> scanner chunk position 

use crate::mesher::MESHER;

use super::math::*;
use spacetimedb::{
    table, ReducerContext, Table,
    //client_visibility_filter, Filter
};
use std::{
    collections::*,
    sync::*,
};
use include_directory::{include_directory, Dir};

pub mod blocks;
mod generate;
use generate::*;

pub use blocks::*;

pub(super) static SCHEME_DIR: Dir<'static> = include_directory!("./scheme");

#[derive(Debug)]
pub struct LoadArea(RwLock<HashMap<IVec3, Arc<Chunk>>>);

impl LoadArea {
    pub fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }

    pub fn insert(&self, position: IVec3, chunk: Arc<Chunk>) {
        let mut access = self.0.write().unwrap();
        access.insert(position, chunk);
    }

    pub fn get(&self, position: IVec3) -> Option<Arc<Chunk>> {
        let access = self.0.read().unwrap();
        access.get(&position).cloned()
    }

    pub fn _remove(&self, position: IVec3) {
        let mut access = self.0.write().unwrap();
        access.remove(&position);
    }
}

// World data in RAM
pub static LOAD_AREA: OnceLock<LoadArea> = OnceLock::new();

// Chunk constants
pub const SIZE: usize = 32;
pub const SIZE_I32: i32 = SIZE as i32;
pub const SIZE_P3: usize = SIZE.pow(3);

// Collection of blocks
#[table(name = chunk, public)]
#[derive(Debug)]
pub struct Chunk {
    #[auto_inc]
    #[primary_key]
    id: u64,

    #[unique]
    pub position: StIVec3,
    // Vec of blocks
    pub blocks: Vec<u16>
}

impl Chunk {
    pub fn empty() -> Vec<u16> {
        return std::iter::repeat_n(0, SIZE_P3).collect();
    }

    pub fn new(position: StIVec3, blocks: Vec<u16>) -> Self {
        Self { id: 0, position, blocks }
    }

    /// XZY coord system
    pub fn block_index(pos: IVec3) -> usize {
        let x = pos.x % SIZE_I32;
        let z = pos.z * SIZE_I32;
        let y = pos.y * SIZE_I32.pow(2);

        (x + y + z) as usize
    }
}

pub fn init_blocks(ctx: &ReducerContext) {
    // clear blocks data
    for block in ctx.db.block().iter() {
        ctx.db.block().id().delete(block.id);
    }

    let blocks_file = SCHEME_DIR.get_file("blocks.json")
        .expect("Blocks data file is not found");
    
    let blocks: Vec<(String, ModelType)> = blocks_file.contents_utf8()
        .and_then(|data| serde_json::from_str(data).ok())
        .expect("Blocks data file parse error");

    for (id, (name, model)) in blocks.into_iter().enumerate() {
        let id = id as u16;
        ctx.db.block().insert(Block { id, name, model });
    }

    BLOCKS_HANDLER.set(BlocksHandler::new(ctx)).unwrap();
}

#[repr(transparent)]
/// Contains all near chunks:
/// 
/// Current; Left; Right; Down; Up; Back; Forward;
pub struct ChunksRefs([Arc<Chunk>; 7]);

impl ChunksRefs {
    // Array of chunk neighbours positions
    pub const OFFSETS: [IVec3; 7] = [
        IVec3::ZERO,  // current
        IVec3::NEG_Y, // down
        IVec3::Y,     // up
        IVec3::NEG_X, // left
        IVec3::X,     // right
        IVec3::NEG_Z, // forward
        IVec3::Z,     // back
    ];

    pub const SIZE: usize = SIZE;
    pub const SIZE_I32: i32 = SIZE_I32;

    // Helper function: create an array from Vec
    fn to_array<T: std::fmt::Debug, const N: usize>(data: Vec<T>) -> [T; N] {
        data.try_into().expect("Wrong size")
    }

    // Helper function: get chunk from BD
    pub fn get_chunk(position: IVec3) -> Option<Arc<Chunk>> {
        LOAD_AREA.get().unwrap().get(position)
    }

    // Create chunk refs
    pub fn new(pos: IVec3) -> Option<Self> {
        let mut data = Vec::<Arc<Chunk>>::with_capacity(7);
        for n in 0..7 {
            data.push(Self::get_chunk(pos + ChunksRefs::OFFSETS[n])?)
        }

        Some(Self(Self::to_array(data)))
    }

    fn offset_index(v: IVec3) -> usize {
        Self::OFFSETS.iter().position(|p| p==&v).unwrap()
    }

    fn chunk_index(x: usize, y: usize, z: usize) -> usize {
        let (cx, cy, cz) = (
            (x / Self::SIZE) as i32,
            (y / Self::SIZE) as i32, 
            (z / Self::SIZE) as i32
        );
        
        Self::offset_index(IVec3::new(cx, cy, cz) - IVec3::ONE)
    }
    
    fn block_index(x: usize, y: usize, z: usize) -> usize {
        let (bx, by, bz) = (
            (x % Self::SIZE) as i32,
            (y % Self::SIZE) as i32,
            (z % Self::SIZE) as i32
        );

        Chunk::block_index(IVec3::new(bx, by, bz))
    }

    pub fn get_block(&self, pos: IVec3) -> u16 {
        let x = (pos.x + Self::SIZE_I32) as usize;
        let y = (pos.y + Self::SIZE_I32) as usize;
        let z = (pos.z + Self::SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        self.0[chunk].blocks[block]
    }
}

pub fn generate_world(ctx: &ReducerContext) {
    LOAD_AREA.set(LoadArea::new()).unwrap();

    let range = 4;
    for x in -range..=range {
        for y in -range..=range {
            for z in -range..=range {
                let pos = ivec3(x, y, z);
                let Some(chunk) = generate_chunk(ctx, pos) else {
                    continue;
                };

                LOAD_AREA.get().unwrap().insert(pos, Arc::new(chunk));
            }
        }
    }

    let mut mesher = MESHER.get().unwrap().write().unwrap();

    let range = 3;
    for x in -range..=range {
        for y in -range..=range {
            for z in -range..=range {
                let pos = ivec3(x, y, z);
                mesher.queue.push(pos);
            }
        }
    }

}