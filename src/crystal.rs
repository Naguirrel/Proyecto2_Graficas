use crate::{
    basis::Basis3,
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
};

const CRYSTAL_EPSILON: f32 = 0.0001;
const TWO_PI: f32 = std::f32::consts::PI * 2.0;
pub const CRYSTAL_MIN_SIDES: u32 = 3;
pub const CRYSTAL_MAX_SIDES: u32 = 12;

/// Faceted crystal: a regular prism with a pyramid on top and, optionally, an
/// inverted pyramid under it.
///
/// In local space the axis is +Y. The prism goes from `y = 0` (the base
/// point) to `y = body_height`, the top apex is at
/// `y = body_height + tip_height` and the bottom apex at
/// `y = -base_tip_height`. A zero tip height gives a flat cap on that end, so
/// `base_tip_height = 0` is a crystal growing out of a surface and
/// `body_height = 0` with both tips is a cut gem (a bipyramid).
///
/// A positive `tip_cut` slices the top pyramid flat that far below its apex,
/// which gives the table of a cut gem.
///
/// The cross-section is a regular polygon with `sides` sides and circumradius
/// `radius`. Face 0 faces local +X and the faces go counterclockwise seen
/// from +Y. Every face is flat, so the crystal shades as facets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Crystal {
    base: Vec3,
    radius: f32,
    body_height: f32,
    tip_height: f32,
    tip_cut: f32,
    base_tip_height: f32,
    sides: u32,
    step_cos: f32,
    step_sin: f32,
    orientation: Basis3,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrystalError {
    NonFiniteBase,
    InvalidRadius,
    InvalidHeight,
    InvalidSides,
}

/// Size and number of sides of a crystal, independent of where it is placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrystalShape {
    /// Circumradius of the polygon cross-section.
    pub radius: f32,
    /// Length of the prism part along the axis.
    pub body_height: f32,
    /// Height of the top pyramid; zero gives a flat top.
    pub tip_height: f32,
    /// Length cut off the top pyramid below its apex, leaving a flat table.
    /// Must be smaller than `tip_height`; zero keeps the point.
    pub tip_cut: f32,
    /// Height of the bottom pyramid; zero gives a flat base.
    pub base_tip_height: f32,
    pub sides: u32,
}

/// Half-space `normal . point <= offset` in local space.
#[derive(Debug, Clone, Copy)]
struct FacePlane {
    normal: Vec3,
    offset: f32,
}

impl Crystal {
    pub fn new(
        base: Vec3,
        shape: CrystalShape,
        orientation: Basis3,
        material_id: usize,
    ) -> Result<Self, CrystalError> {
        let CrystalShape {
            radius,
            body_height,
            tip_height,
            tip_cut,
            base_tip_height,
            sides,
        } = shape;

        if !vec3_is_finite(base) {
            return Err(CrystalError::NonFiniteBase);
        }

        if !radius.is_finite() || radius <= 0.0 {
            return Err(CrystalError::InvalidRadius);
        }

        let heights = [body_height, tip_height, base_tip_height];
        if heights
            .iter()
            .any(|height| !height.is_finite() || *height < 0.0)
            || heights.iter().sum::<f32>() <= CRYSTAL_EPSILON
            || !tip_cut.is_finite()
            || tip_cut < 0.0
            || (tip_cut > 0.0 && tip_cut >= tip_height)
        {
            return Err(CrystalError::InvalidHeight);
        }

        if !(CRYSTAL_MIN_SIDES..=CRYSTAL_MAX_SIDES).contains(&sides) {
            return Err(CrystalError::InvalidSides);
        }

        let (step_sin, step_cos) = (TWO_PI / sides as f32).sin_cos();

        Ok(Self {
            base,
            radius,
            body_height,
            tip_height,
            tip_cut,
            base_tip_height,
            sides,
            step_cos,
            step_sin,
            orientation,
            material_id,
        })
    }

    /// Point on the axis where the prism starts (`y = 0` in local space).
    pub fn base(self) -> Vec3 {
        self.base
    }

    pub fn radius(self) -> f32 {
        self.radius
    }

    pub fn body_height(self) -> f32 {
        self.body_height
    }

    pub fn tip_height(self) -> f32 {
        self.tip_height
    }

    pub fn tip_cut(self) -> f32 {
        self.tip_cut
    }

    pub fn base_tip_height(self) -> f32 {
        self.base_tip_height
    }

    pub fn sides(self) -> u32 {
        self.sides
    }

    pub fn orientation(self) -> Basis3 {
        self.orientation
    }

    pub fn material_id(self) -> usize {
        self.material_id
    }

    /// Height of the highest point above the base point.
    fn top(self) -> f32 {
        self.body_height + self.tip_height - self.tip_cut
    }

    /// Length along the axis from the bottom end to the top.
    pub fn total_height(self) -> f32 {
        self.base_tip_height + self.top()
    }

    /// World position of the top apex, or of the center of the flat top.
    pub fn apex(self) -> Vec3 {
        self.local_to_world_point(Vec3::new(0.0, self.top(), 0.0))
    }

    /// Center of the local bounding box, in world space.
    pub fn center(self) -> Vec3 {
        let middle = (self.top() - self.base_tip_height) * 0.5;

        self.local_to_world_point(Vec3::new(0.0, middle, 0.0))
    }

    /// Half extents of the local bounding box (radius, half height, radius).
    pub fn half_extents(self) -> Vec3 {
        Vec3::new(self.radius, self.total_height() * 0.5, self.radius)
    }

    /// Distance from the axis to the middle of each side face.
    pub fn apothem(self) -> f32 {
        self.radius * (std::f32::consts::PI / self.sides as f32).cos()
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        let local = self.world_to_local_point(point);
        let mut inside = true;

        self.for_each_plane(|plane| {
            inside &= plane.normal.dot(local) <= plane.offset + CRYSTAL_EPSILON;
        });

        inside
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !vec3_is_finite(ray.origin)
            || !vec3_is_finite(ray.direction)
            || ray.direction == Vec3::ZERO
            || !t_min.is_finite()
            || !t_max.is_finite()
            || t_min > t_max
        {
            return None;
        }

        let origin = self.world_to_local_point(ray.origin);
        let direction = self.orientation.world_to_local_vector(ray.direction);
        let mut enter = f32::NEG_INFINITY;
        let mut exit = f32::INFINITY;
        let mut enter_normal = Vec3::ZERO;
        let mut exit_normal = Vec3::ZERO;
        let mut outside_parallel = false;

        // Clips the ray against every face plane of the convex solid.
        self.for_each_plane(|plane| {
            let distance_to_plane = plane.offset - plane.normal.dot(origin);
            let approach = plane.normal.dot(direction);

            if approach.abs() <= f32::EPSILON {
                outside_parallel |= distance_to_plane < 0.0;
                return;
            }

            let distance = distance_to_plane / approach;
            if approach < 0.0 {
                if distance > enter {
                    enter = distance;
                    enter_normal = plane.normal;
                }
            } else if distance < exit {
                exit = distance;
                exit_normal = plane.normal;
            }
        });

        if outside_parallel || enter > exit {
            return None;
        }

        let (distance, local_normal) = if enter >= t_min && enter <= t_max {
            (enter, enter_normal)
        } else if exit >= t_min && exit <= t_max {
            (exit, exit_normal)
        } else {
            return None;
        };

        let local_position = origin + direction * distance;
        let position = self.local_to_world_point(local_position);
        let normal = self
            .orientation
            .local_to_world_vector(local_normal)
            .normalized();

        if !distance.is_finite()
            || !vec3_is_finite(position)
            || !vec3_is_finite(normal)
            || normal == Vec3::ZERO
        {
            return None;
        }

        Some(Intersection::new(
            distance,
            position,
            normal,
            self.local_uv(local_position),
            self.material_id,
        ))
    }

    /// Calls `visit` with every face plane: the flat caps first, then the
    /// side, top and bottom faces of each side of the polygon.
    fn for_each_plane(&self, mut visit: impl FnMut(FacePlane)) {
        let apothem = self.apothem();
        let body_top = self.body_height;

        if self.tip_height <= 0.0 || self.tip_cut > 0.0 {
            visit(FacePlane {
                normal: Vec3::new(0.0, 1.0, 0.0),
                offset: self.top(),
            });
        }

        if self.base_tip_height <= 0.0 {
            visit(FacePlane {
                normal: Vec3::new(0.0, -1.0, 0.0),
                offset: 0.0,
            });
        }

        let (mut face_cos, mut face_sin) = (1.0_f32, 0.0_f32);
        for _ in 0..self.sides {
            let outward = Vec3::new(face_cos, 0.0, face_sin);

            if self.body_height > 0.0 {
                visit(FacePlane {
                    normal: outward,
                    offset: apothem,
                });
            }

            // Each pyramid face goes from the prism edge at the apothem to the
            // apex on the axis.
            if self.tip_height > 0.0 {
                let slope = apothem / self.tip_height;
                visit(FacePlane {
                    normal: Vec3::new(outward.x, slope, outward.z),
                    offset: apothem + slope * body_top,
                });
            }

            if self.base_tip_height > 0.0 {
                let slope = apothem / self.base_tip_height;
                visit(FacePlane {
                    normal: Vec3::new(outward.x, -slope, outward.z),
                    offset: apothem,
                });
            }

            (face_cos, face_sin) = (
                face_cos * self.step_cos - face_sin * self.step_sin,
                face_sin * self.step_cos + face_cos * self.step_sin,
            );
        }
    }

    fn local_uv(&self, position: Vec3) -> Vec2 {
        let around = (position.z.atan2(position.x) / TWO_PI).rem_euclid(1.0);
        let along = (position.y + self.base_tip_height) / self.total_height();

        Vec2::new(
            if around < 1.0 { around } else { 0.0 },
            along.clamp(0.0, 1.0),
        )
    }

    fn local_to_world_point(self, point: Vec3) -> Vec3 {
        self.base + self.orientation.local_to_world_vector(point)
    }

    fn world_to_local_point(self, point: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(point - self.base)
    }
}

fn vec3_is_finite(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{CRYSTAL_MAX_SIDES, Crystal, CrystalError, CrystalShape};
    use crate::{basis::Basis3, math::Vec3, ray::Ray};

    fn shape(
        radius: f32,
        body_height: f32,
        tip_height: f32,
        base_tip_height: f32,
        sides: u32,
    ) -> CrystalShape {
        CrystalShape {
            radius,
            body_height,
            tip_height,
            tip_cut: 0.0,
            base_tip_height,
            sides,
        }
    }

    /// Hexagonal crystal with a flat base at the origin, a unit tall prism
    /// and a half unit tall tip.
    fn hexagonal_crystal() -> Crystal {
        Crystal::new(
            Vec3::ZERO,
            shape(0.5, 1.0, 0.5, 0.0, 6),
            Basis3::identity(),
            4,
        )
        .unwrap()
    }

    fn gem() -> Crystal {
        Crystal::new(
            Vec3::ZERO,
            shape(0.5, 0.0, 0.4, 0.6, 8),
            Basis3::identity(),
            2,
        )
        .unwrap()
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0005, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!((left - right).length() < 0.0005, "{left:?} != {right:?}");
    }

    #[test]
    fn constructor_rejects_invalid_geometry() {
        let basis = Basis3::identity();

        assert_eq!(
            Crystal::new(
                Vec3::new(f32::NAN, 0.0, 0.0),
                shape(1.0, 1.0, 1.0, 0.0, 6),
                basis,
                0
            ),
            Err(CrystalError::NonFiniteBase)
        );
        assert_eq!(
            Crystal::new(Vec3::ZERO, shape(0.0, 1.0, 1.0, 0.0, 6), basis, 0),
            Err(CrystalError::InvalidRadius)
        );
        assert_eq!(
            Crystal::new(Vec3::ZERO, shape(1.0, -1.0, 1.0, 0.0, 6), basis, 0),
            Err(CrystalError::InvalidHeight)
        );
        assert_eq!(
            Crystal::new(Vec3::ZERO, shape(1.0, 0.0, 0.0, 0.0, 6), basis, 0),
            Err(CrystalError::InvalidHeight)
        );
        assert_eq!(
            Crystal::new(Vec3::ZERO, shape(1.0, 1.0, f32::INFINITY, 0.0, 6), basis, 0),
            Err(CrystalError::InvalidHeight)
        );
        assert_eq!(
            Crystal::new(Vec3::ZERO, shape(1.0, 1.0, 1.0, 0.0, 2), basis, 0),
            Err(CrystalError::InvalidSides)
        );
        assert_eq!(
            Crystal::new(
                Vec3::ZERO,
                shape(1.0, 1.0, 1.0, 0.0, CRYSTAL_MAX_SIDES + 1),
                basis,
                0
            ),
            Err(CrystalError::InvalidSides)
        );
    }

    #[test]
    fn accessors_describe_the_crystal() {
        let crystal = hexagonal_crystal();

        assert_eq!(crystal.base(), Vec3::ZERO);
        assert_eq!(crystal.radius(), 0.5);
        assert_eq!(crystal.sides(), 6);
        assert_eq!(crystal.material_id(), 4);
        assert_near(crystal.total_height(), 1.5);
        assert_near(crystal.apothem(), 0.5 * (std::f32::consts::PI / 6.0).cos());
        assert_vec_near(crystal.apex(), Vec3::new(0.0, 1.5, 0.0));
        assert_vec_near(crystal.center(), Vec3::new(0.0, 0.75, 0.0));
        assert_vec_near(crystal.half_extents(), Vec3::new(0.5, 0.75, 0.5));
    }

    #[test]
    fn side_hit_uses_the_flat_face_normal() {
        let crystal = hexagonal_crystal();
        let ray = Ray::new(Vec3::new(3.0, 0.5, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = crystal.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 3.0 - crystal.apothem());
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(hit.material_id, 4);
    }

    #[test]
    fn nearby_rays_on_one_face_share_the_same_normal() {
        let crystal = hexagonal_crystal();
        let first = crystal
            .intersect(
                &Ray::new(Vec3::new(3.0, 0.3, -0.05), Vec3::new(-1.0, 0.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();
        let second = crystal
            .intersect(
                &Ray::new(Vec3::new(3.0, 0.8, 0.12), Vec3::new(-1.0, 0.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();

        assert_vec_near(first.normal, second.normal);
    }

    #[test]
    fn tip_hit_tilts_the_normal_up_and_out() {
        let crystal = hexagonal_crystal();
        let ray = Ray::new(Vec3::new(0.1, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = crystal.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.position.y > 1.0 && hit.position.y < 1.5);
        assert!(hit.normal.y > 0.0);
        assert!(hit.normal.x > 0.0);
        assert!(crystal.contains_point(hit.position - hit.normal * 0.001));
    }

    #[test]
    fn flat_base_is_hit_from_below() {
        let crystal = hexagonal_crystal();
        let ray = Ray::new(Vec3::new(0.0, -2.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = crystal.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 2.0);
        assert_vec_near(hit.normal, Vec3::new(0.0, -1.0, 0.0));
    }

    #[test]
    fn gem_has_pointed_ends_on_both_sides() {
        let gem = gem();
        let from_below = gem
            .intersect(
                &Ray::new(Vec3::new(0.0, -3.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();
        let from_above = gem
            .intersect(
                &Ray::new(Vec3::new(0.0, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();

        assert_near(from_below.position.y, -0.6);
        assert_near(from_above.position.y, 0.4);
        assert!(gem.contains_point(Vec3::ZERO));
        assert!(!gem.contains_point(Vec3::new(0.4, 0.3, 0.0)));
    }

    #[test]
    fn tip_cut_leaves_a_flat_table() {
        let cut_gem = Crystal::new(
            Vec3::ZERO,
            CrystalShape {
                tip_cut: 0.2,
                ..shape(0.5, 0.1, 0.4, 0.6, 8)
            },
            Basis3::identity(),
            0,
        )
        .unwrap();
        let from_above = cut_gem
            .intersect(
                &Ray::new(Vec3::new(0.05, 3.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
                0.001,
                100.0,
            )
            .unwrap();

        assert_near(from_above.position.y, 0.3);
        assert_vec_near(from_above.normal, Vec3::new(0.0, 1.0, 0.0));
        assert_vec_near(cut_gem.apex(), Vec3::new(0.0, 0.3, 0.0));
        assert_near(cut_gem.total_height(), 0.9);
        assert_eq!(
            Crystal::new(
                Vec3::ZERO,
                CrystalShape {
                    tip_cut: 0.4,
                    ..shape(0.5, 0.1, 0.4, 0.6, 8)
                },
                Basis3::identity(),
                0,
            ),
            Err(CrystalError::InvalidHeight)
        );
    }

    #[test]
    fn rays_that_pass_beside_or_behind_miss() {
        let crystal = hexagonal_crystal();

        assert!(
            crystal
                .intersect(
                    &Ray::new(Vec3::new(3.0, 0.5, 0.6), Vec3::new(-1.0, 0.0, 0.0)),
                    0.001,
                    100.0
                )
                .is_none()
        );
        assert!(
            crystal
                .intersect(
                    &Ray::new(Vec3::new(0.0, 1.6, 0.0), Vec3::new(1.0, 0.0, 0.0)),
                    0.001,
                    100.0
                )
                .is_none()
        );
        assert!(
            crystal
                .intersect(
                    &Ray::new(Vec3::new(3.0, 0.5, 0.0), Vec3::new(1.0, 0.0, 0.0)),
                    0.001,
                    100.0
                )
                .is_none()
        );
        assert!(
            crystal
                .intersect(
                    &Ray::new(Vec3::new(3.0, 0.5, 0.0), Vec3::new(-1.0, 0.0, 0.0)),
                    0.001,
                    1.0
                )
                .is_none()
        );
    }

    #[test]
    fn ray_from_inside_hits_the_exit_face_with_outward_normal() {
        let crystal = hexagonal_crystal();
        let ray = Ray::new(Vec3::new(0.0, 0.5, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let hit = crystal.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, crystal.apothem());
        assert_vec_near(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn orientation_and_base_move_the_crystal() {
        let orientation = Basis3::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let crystal = Crystal::new(
            Vec3::new(1.0, 2.0, 0.0),
            shape(0.5, 1.0, 0.5, 0.0, 6),
            orientation,
            0,
        )
        .unwrap();
        let ray = Ray::new(Vec3::new(-5.0, 2.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        let hit = crystal.intersect(&ray, 0.001, 100.0).unwrap();

        assert_vec_near(crystal.apex(), Vec3::new(-0.5, 2.0, 0.0));
        assert_near(hit.position.x, -0.5);
        // The ray meets the apex, so the normal is one of the tip faces.
        assert!(hit.normal.x < -0.5);
    }

    #[test]
    fn uv_stays_in_range() {
        let crystal = hexagonal_crystal();

        for index in 0..24 {
            let angle = index as f32 / 24.0 * std::f32::consts::TAU;
            let direction = Vec3::new(-angle.cos(), -0.2, -angle.sin());
            let ray = Ray::new(
                Vec3::new(angle.cos() * 4.0, 1.2, angle.sin() * 4.0),
                direction,
            );

            if let Some(hit) = crystal.intersect(&ray, 0.001, 100.0) {
                assert!((0.0..1.0).contains(&hit.uv.u));
                assert!((0.0..=1.0).contains(&hit.uv.v));
            }
        }
    }
}
