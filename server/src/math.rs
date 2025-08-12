pub use bevy_math::*;
use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StIVec3 {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<IVec3> for StIVec3 {
    fn from(value: IVec3) -> Self {
        StIVec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<StIVec3> for IVec3 {
    fn from(value: StIVec3) -> Self {
        IVec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq)]
pub struct StVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for StVec3 {
    fn from(value: Vec3) -> Self {
        StVec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<StVec3> for Vec3 {
    fn from(value: StVec3) -> Self {
        Vec3 {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
