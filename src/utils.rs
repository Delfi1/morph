pub const SIZE: usize = 16;
pub const SIZE_F32: f32 = SIZE as f32;

use super::stdb;
pub use bevy::math::*;

impl From<stdb::StIVec3> for IVec3 {
    fn from(value: stdb::StIVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<stdb::StVec3> for Vec3 {
    fn from(value: stdb::StVec3) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}