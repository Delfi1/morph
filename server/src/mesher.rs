use super::{
    math::*,
    chunks::Chunk
};
use spacetimedb::{table};

// todo: mesh lods

#[table(name = mesh, public)]
/// Mesh table (or cached mesh)
pub struct Mesh {
    #[unique]
    position: StIVec3,
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

/// Mesh builder
pub fn _build(_chunk: &Chunk) -> Mesh {
    todo!();
}
