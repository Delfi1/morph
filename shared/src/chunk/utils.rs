pub(super) use crate::math::*;

/// Bevy IVec3's representation in Rune
#[derive(rune::Any, Clone, Copy)]
pub struct RnIVec3(pub IVec3);

impl RnIVec3 {
    pub fn new(position: IVec3) -> Self {
        Self(position)
    }
}

#[rune::function]
pub fn ivec3(x: i32, y: i32, z: i32) -> RnIVec3 {
    RnIVec3(IVec3::new(x, y, z))
}
