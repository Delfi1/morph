mod utils;
pub use utils::*;
use std::sync::*;

/// Chunk size
pub const SIZE: usize = 16;
pub const SIZE_I32: i32 = SIZE as i32;
pub const SIZE_P3: usize = SIZE.pow(3);

// Block size in bytes and chunk buffer (data) size
pub const BLOCK_SIZE: usize = 12;
pub const BYTE: usize = 8;
pub const HALF_BYTE: usize = BYTE / 2;

// How much bytes
pub const BUF_SIZE: usize = SIZE_P3 * BLOCK_SIZE / BYTE;

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct RawChunk(Vec<u8>);

impl RawChunk {
    // New empty chunk
    pub fn empty() -> Self {
        Self(std::iter::repeat_n(0, BUF_SIZE).collect())
    }

    pub fn get_block(&self, index: usize) -> u16 {
        let i = index * BLOCK_SIZE / BYTE;
        let (a, b) = match index % 2 == 0 {
            // First and second bytes
            // 0110_0001 1001_0010 1110_1000 => 0110_0001 and 1001
            true => (self.0[i] as u16, (self.0[i + 1] >> HALF_BYTE) as u16),
            // Second and third bytes
            // 0110_0001 1001_0010 1110_1000 => 0010 and 1110_1000
            false => ((self.0[i] << HALF_BYTE) as u16, self.0[i + 1] as u16),
        };

        a << HALF_BYTE | b
    }

    pub fn set_block(&mut self, index: usize, value: u16) {
        let i = index * BLOCK_SIZE / BYTE;
        if index % 2 == 0 {
            // 0000_0101_1100_0011 => 0101_1100 and 0011
            let (a, b) = ((value >> HALF_BYTE) as u8, (value & 0b1111) as u8);

            self.0[i] = a;
            self.0[i + 1] = (self.0[i + 1] & 0b0000_1111) | (b << HALF_BYTE);
        } else {
            // 0000_0101_1100_0011 => 0101 and 1100_0011
            let (a, b) = ((value >> BYTE) as u8, (value & 0b1111_1111) as u8);

            self.0[i] = (self.0[i] & 0b1111_0000) | a;
            self.0[i + 1] = b;
        }
    }

    pub fn block_index(pos: IVec3) -> usize {
        let x = pos.x % SIZE_I32;
        let z = pos.z * SIZE_I32;
        let y = pos.y * SIZE_I32.pow(2);

        (x + y + z) as usize
    }
}

#[derive(Debug, rune::Any, Clone)]
pub struct Chunk(Arc<RwLock<RawChunk>>);

impl Chunk {
    pub fn empty() -> Self {
        Self::new(RawChunk::empty())
    }

    pub fn new(raw: RawChunk) -> Self {
        Self(Arc::new(RwLock::new(raw)))
    }

    pub fn read(&self) -> RwLockReadGuard<'_, RawChunk> {
        self.0.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, RawChunk> {
        self.0.write().unwrap()
    }
}

#[repr(transparent)]
#[derive(rune::Any)]
/// Current; Left; Right; Down; Up; Back; Forward;
pub struct ChunksRefs([Chunk; 7]);

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

    // Helper function: create an array from Vec
    fn to_array<T: std::fmt::Debug, const N: usize>(data: Vec<T>) -> [T; N] {
        data.try_into().expect("Wrong size")
    }

    // Create chunk refs
    pub fn new(pos: IVec3) -> Option<Self> {
        let mut data = Vec::with_capacity(7);
        for n in 0..7 {
            data.push(super::get_chunk(pos + ChunksRefs::OFFSETS[n])?)
        }

        Some(Self(Self::to_array(data)))
    }

    fn offset_index(v: IVec3) -> usize {
        Self::OFFSETS.iter().position(|p| p == &v).unwrap()
    }

    fn chunk_index(x: usize, y: usize, z: usize) -> usize {
        let (cx, cy, cz) = ((x / SIZE) as i32, (y / SIZE) as i32, (z / SIZE) as i32);

        Self::offset_index(IVec3::new(cx, cy, cz) - IVec3::ONE)
    }

    fn block_index(x: usize, y: usize, z: usize) -> usize {
        let (bx, by, bz) = ((x % SIZE) as i32, (y % SIZE) as i32, (z % SIZE) as i32);

        RawChunk::block_index(IVec3::new(bx, by, bz))
    }

    pub fn get_block(&self, pos: IVec3) -> u16 {
        let x = (pos.x + SIZE_I32) as usize;
        let y = (pos.y + SIZE_I32) as usize;
        let z = (pos.z + SIZE_I32) as usize;
        let chunk = Self::chunk_index(x, y, z);
        let block = Self::block_index(x, y, z);

        RawChunk::get_block(&self.0[chunk].read(), block)
    }
}

// ----------------------------------------------------------------------------------------------
// Chunks functions

#[rune::function]
pub fn get_block(chunk: &RnIVec3, block: &RnIVec3) -> u16 {
    let chunk = super::get_chunk(chunk.0).unwrap();
    let data = chunk.read();

    data.get_block(RawChunk::block_index(block.0))
}

#[rune::function]
pub fn set_block(chunk: &RnIVec3, block: &RnIVec3, value: u16) {
    let chunk = super::get_chunk(chunk.0).unwrap();
    let mut data = chunk.write();

    data.set_block(RawChunk::block_index(block.0), value)
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