use crate::math::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intersection {
    pub distance: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
    pub material_id: usize,
}

impl Intersection {
    pub fn new(distance: f32, position: Vec3, normal: Vec3, uv: Vec2, material_id: usize) -> Self {
        Self {
            distance,
            position,
            normal,
            uv,
            material_id,
        }
    }
}
