use std::{
    collections::*,
    sync::*
};

use super::{
    math::*,
    chunks::{SIZE_I32, ChunksRefs, is_meshable}
};
use bevy_tasks::{block_on, AsyncComputeTaskPool, Task};
use log::info;
use spacetimedb::{
    table, ReducerContext, Table,
    //Filter, client_visibility_filter
};

#[derive(Debug)]
pub struct Mesher {
    pub queue: Vec<IVec3>,
    pub tasks: HashMap<IVec3, Task<Mesh>>
}

impl Mesher {
    pub const MAX_TASKS: usize = 16;

    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            tasks: HashMap::new()
        }
    }
}

pub static MESHER: OnceLock<RwLock<Mesher>> = OnceLock::new();

// Also face normal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left, Right, Down, Up, Back, Forward
}

impl Direction {
    /// Get block position from grid and axis
    pub fn world_sample(&self, axis: i32, row: i32, column: i32) -> IVec3 {
        match self {
            Self::Up => IVec3::new(row, axis-1, column),
            Self::Down => IVec3::new(row, axis, column),
            Self::Left => IVec3::new(axis, column, row),
            Self::Right => IVec3::new(axis-1, column, row),
            Self::Forward => IVec3::new(row, column, axis),
            Self::Back => IVec3::new(row, column, axis-1),
        }
    } 

    /// Get next -Z block relative pos
    pub fn air_sample(&self) -> IVec3 {
        match self {
            Self::Up => IVec3::Y,
            Self::Down => IVec3::NEG_Y,
            Self::Left => IVec3::NEG_X,
            Self::Right => IVec3::X,
            Self::Forward => IVec3::NEG_Z,
            Self::Back => IVec3::Z,
        }
    }

    pub fn reverse_order(&self) -> bool {
        match self {
            Self::Up => true,
            Self::Down => false,
            Self::Left => false,
            Self::Right => true,
            Self::Forward => true,
            Self::Back => false,
        }
    }

    pub fn negate_axis(&self) -> i32 {
        match self {
            Self::Up | Self::Right | Self::Back => 1,
            _ => 0
        }
    }
    
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Up => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Forward => 3,
            Self::Back => 4,
            Self::Down => 5,
        }
    }

    pub fn iter() -> Vec<Self> {
        vec![Self::Left, Self::Right, Self::Down, Self::Up, Self::Back, Self::Forward]
    }
}

pub struct Face { x: i32, y: i32 }

/// All blocks face methods
impl Face {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// UV corners
    pub const UVS: [UVec2; 4] = [
        UVec2::new(1, 1),
        UVec2::new(0, 1),
        UVec2::new(0, 0),
        UVec2::new(1, 0)
    ];

    /// Make vertices from face
    pub fn vertices(self, dir: Direction, mut axis: i32, block: u16) -> Vec<u32> {
        axis += dir.negate_axis();
        let v1 = Vertex::new(
            dir.world_sample(axis, self.x, self.y), 
            dir,
            block as u32,
            &Self::UVS[0]
        );

        let v2 = Vertex::new(
            dir.world_sample(axis, self.x + 1, self.y), 
            dir,
            block as u32,
            &Self::UVS[1]
        );

        let v3 = Vertex::new(
            dir.world_sample(axis, self.x + 1, self.y + 1), 
            dir,
            block as u32,
            &Self::UVS[2]
        );

        let v4 = Vertex::new(
            dir.world_sample(axis, self.x, self.y + 1), 
            dir,
            block as u32,
            &Self::UVS[3]
        );
        
        let mut new = std::collections::VecDeque::from([v1, v2, v3, v4]);
        if dir.reverse_order() {
            let o = new.split_off(1);
            o.into_iter().rev().for_each(|i| new.push_back(i));
        }

        Vec::from(new)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex;
impl Vertex {
    /// Pocket of vertex data
    /// [6]bits - X (0-63)
    /// [6]bits - Y (0-63)
    /// [6]bits - Z (0-63)
    /// [3]bits - Face (0-7)
    /// [7]bits - texture_x (0-255)
    /// [1]bit - UVx (0/1)
    /// [1]bit - UVy (0/1)
    pub fn new(local: IVec3, dir: Direction, block: u32, uv: &UVec2) -> u32 {
        let data = local.x as u32
        | (local.y as u32) << 6u32
        | (local.z as u32) << 12u32
        | (dir.to_u32()) << 18u32
        | (block) << 21u32 // Block id also texture id in binding array 
        | (uv.x) << 28u32  // UV may be only 0 or 1
        | (uv.y) << 29u32;
        
        data
    }
}

#[table(name = mesh, public)]
#[derive(Debug)]
/// Mesh table (or cached mesh)
pub struct Mesh {
    #[auto_inc]
    #[primary_key]
    id: u64,

    #[unique]
    position: StIVec3,
    vertices: Vec<u32>,
    indices: Vec<u32>,
}

impl Mesh {
    fn make_vertices(dir: Direction, refs: &ChunksRefs) -> Vec<u32> {
        let mut vertices = Vec::with_capacity(512);
        let size = SIZE_I32;

        // Culled meshser
        for axis in 0..size {
            for i in 0..size.pow(2) {
                let row = i % size;
                let column = i / size;
                let pos = dir.world_sample(axis, row, column);
                let (current, neg_z) =
                    (refs.get_block(pos), refs.get_block(pos + dir.air_sample()));

                if is_meshable(current) && !is_meshable(neg_z) {
                    let face = Face::new(row, column);
                    vertices.extend(face.vertices(dir, axis, current));
                }
            }
        }

        vertices
    }

    pub fn build(refs: ChunksRefs) -> Vec<u32> {
        let mut vertices = Vec::new();

        // Apply all directions
        for dir in Direction::iter() {
            vertices.extend(Self::make_vertices(dir, &refs));
        }
        
        vertices
    }

    pub fn generate_indices(vertices: &Vec<u32>) -> Vec<u32> {
        let indices_count = vertices.len() / 4;
        let mut indices = Vec::<u32>::with_capacity(indices_count);
        
        (0..indices_count).into_iter().for_each(|vert_index| {
            let vert_index = vert_index as u32 * 4u32;
            indices.push(vert_index);
            indices.push(vert_index + 1);
            indices.push(vert_index + 2);
            indices.push(vert_index);
            indices.push(vert_index + 2);
            indices.push(vert_index + 3);
        });

        indices
    }
}

pub async fn build_mesh(pos: IVec3, refs: ChunksRefs) -> Mesh {
    let vertices = Mesh::build(refs);
    let indices = Mesh::generate_indices(&vertices);

    Mesh {
        id: 0,
        position: pos.into(),
        vertices,
        indices
    }
}

pub fn init_mesher() {
    MESHER.set(RwLock::new(Mesher::new())).unwrap();
}

pub fn proceed_mesher(ctx: &ReducerContext) {
    let task_pool = AsyncComputeTaskPool::get();
    let mut mesher = MESHER.get().unwrap().write().unwrap();

    let l = mesher.queue.len().min(Mesher::MAX_TASKS - mesher.tasks.len());

    for pos in mesher.queue.drain(0..l).collect::<Vec<IVec3>>() {
        let Some(refs) = ChunksRefs::new(pos) else {
            mesher.queue.push(pos);
            continue;
        };

        let task = task_pool.spawn(build_mesh(pos, refs));
        mesher.tasks.insert(pos, task);
    }

    for (pos, task) in mesher.tasks.drain().collect::<Vec<_>>() {
        if !task.is_finished() {
            mesher.tasks.insert(pos, task);
            continue;
        }

        let mesh = block_on(task);
        info!("Generated mesh: {}", pos);
        ctx.db.mesh().insert(mesh);
    }
}