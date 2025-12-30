use super::{Chunk, Core, RawChunk, SIZE, SIZE_I32, math::*};
use std::sync::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    Back,
    Forward,
}

pub struct Mesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

/// Init Rune command
pub async fn build(refs: ChunksRefs) {}

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
            data.push(Core::get_chunk(pos + ChunksRefs::OFFSETS[n])?)
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
