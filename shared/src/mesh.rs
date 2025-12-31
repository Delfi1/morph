use super::math::*;

#[derive(rune::Any, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    Back,
    Forward,
}

#[derive(rune::Any, Debug, Clone)]
pub struct Mesh {
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

/// Create new mesh buffer
#[rune::function]
pub fn new_mesh() -> Mesh {
    Mesh {
        vertices: Vec::with_capacity(256),
        indices: Vec::new()
    }
}

#[rune::function(instance)]
pub fn push_vertex(mesh: &mut Mesh, vertex: u32) {
    mesh.vertices.push(vertex);
}

#[rune::function(instance)]
pub fn push_index(mesh: &mut Mesh, index: u32) {
    mesh.indices.push(index);
}
