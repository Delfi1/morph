use super::chunks::Chunk;
use spacetimedb::table;

#[table(name = mesh, public)]
/// Mesh table (or cached mesh)
pub struct Mesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

/// Mesh builder
pub fn _build(_chunk: &Chunk) -> Mesh {
    todo!();
}
