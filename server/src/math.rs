use bevy_math::*;
use spacetimedb::SpacetimeType;

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq)]
pub struct StIVec3 {
    x: i32,
    y: i32,
    z: i32,
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