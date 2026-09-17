use crate::math::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub material_id: usize,
}

impl Intersection {
    pub fn new(distance: f32, position: Vec3, normal: Vec3, material_id: usize) -> Self {
        Self {
            distance,
            position,
            normal,
            material_id,
        }
    }
}
