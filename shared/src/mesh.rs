use super::math::*;

#[derive(rune::Any, Debug, Clone)]
pub struct Mesh {
    #[rune(set)]
    vertices: Vec<u32>,
    #[rune(set)]
    indices: Vec<u32>,
}

/// Create new mesh buffer
#[rune::function]
pub fn new_mesh() -> Mesh {
    Mesh {
        vertices: Vec::new(),
        indices: Vec::new()
    }
}

