use crate::{
    basis::Basis3,
    intersection::Intersection,
    math::{Vec2, Vec3},
    ray::Ray,
    sphere::Sphere,
};

const CURVED_TETRAHEDRON_EPSILON: f32 = 0.0001;
const VERTEX_COUNT: usize = 4;

/// Face sphere radius, relative to the edge length, of a Reuleaux tetrahedron.
pub const REULEAUX_FACE_RADIUS_SCALE: f32 = 1.0;

/// Regular tetrahedron with curved faces.
///
/// The solid is the intersection of four balls, one per face. Each ball passes
/// through the three vertices of its face and its center lies on the side of
/// the opposite vertex, so the face bulges outward as a piece of a sphere.
///
/// `face_radius_scale` is the radius of those spheres relative to the edge
/// length. With `REULEAUX_FACE_RADIUS_SCALE` (1.0) each ball is centered at
/// the opposite vertex and the solid is a Reuleaux tetrahedron; larger values
/// give flatter faces, closer to a regular tetrahedron.
///
/// In local space the apex vertex points along +Y, the other three vertices
/// lie on the plane `y = -circumradius / 3`, and one of them points along +Z.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurvedTetrahedron {
    center: Vec3,
    circumradius: f32,
    face_radius_scale: f32,
    orientation: Basis3,
    material_id: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurvedTetrahedronError {
    NonFiniteCenter,
    InvalidCircumradius,
    InvalidFaceRadiusScale,
}

impl CurvedTetrahedron {
    pub fn new(
        center: Vec3,
        circumradius: f32,
        face_radius_scale: f32,
        orientation: Basis3,
        material_id: usize,
    ) -> Result<Self, CurvedTetrahedronError> {
        if !vec3_is_finite(center) {
            return Err(CurvedTetrahedronError::NonFiniteCenter);
        }

        if !circumradius.is_finite() || circumradius <= 0.0 {
            return Err(CurvedTetrahedronError::InvalidCircumradius);
        }

        if !face_radius_scale.is_finite() || face_radius_scale < REULEAUX_FACE_RADIUS_SCALE {
            return Err(CurvedTetrahedronError::InvalidFaceRadiusScale);
        }

        Ok(Self {
            center,
            circumradius,
            face_radius_scale,
            orientation,
            material_id,
        })
    }

    pub fn center(self) -> Vec3 {
        self.center
    }

    /// Distance from the center to each vertex.
    pub fn circumradius(self) -> f32 {
        self.circumradius
    }

    /// Edge length of the underlying regular tetrahedron.
    pub fn edge_length(self) -> f32 {
        self.circumradius * (8.0_f32 / 3.0).sqrt()
    }

    pub fn face_radius_scale(self) -> f32 {
        self.face_radius_scale
    }

    /// Radius of each of the four spheres that form the faces.
    pub fn face_radius(self) -> f32 {
        self.edge_length() * self.face_radius_scale
    }

    pub fn orientation(self) -> Basis3 {
        self.orientation
    }

    pub fn material_id(self) -> usize {
        self.material_id
    }

    /// World-space vertices: the apex first, then the three base vertices.
    pub fn vertices(self) -> [Vec3; VERTEX_COUNT] {
        self.local_vertices()
            .map(|vertex| self.center + self.orientation.local_to_world_vector(vertex))
    }

    pub fn contains_point(self, point: Vec3) -> bool {
        let local = self.world_to_local_point(point);
        let limit = self.face_radius() + CURVED_TETRAHEDRON_EPSILON;

        self.local_face_centers()
            .iter()
            .all(|face_center| (local - *face_center).length() <= limit)
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        if !self.has_valid_geometry()
            || !vec3_is_finite(ray.origin)
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
        let a = direction.dot(direction);

        if !a.is_finite() || a <= CURVED_TETRAHEDRON_EPSILON {
            return None;
        }

        let face_centers = self.local_face_centers();
        let radius = self.face_radius();
        let radius_squared = radius * radius;
        // The ray is inside the solid on [enter, exit]: the latest entry and
        // the earliest exit among the four balls.
        let mut enter = (f32::NEG_INFINITY, 0);
        let mut exit = (f32::INFINITY, 0);

        for (index, face_center) in face_centers.iter().enumerate() {
            let offset = origin - *face_center;
            let half_b = offset.dot(direction);
            let c = offset.dot(offset) - radius_squared;
            let discriminant = half_b * half_b - a * c;

            if !discriminant.is_finite() || discriminant < 0.0 {
                return None;
            }

            let root = discriminant.sqrt();
            let near = (-half_b - root) / a;
            let far = (-half_b + root) / a;

            if near > enter.0 {
                enter = (near, index);
            }

            if far < exit.0 {
                exit = (far, index);
            }
        }

        if enter.0 > exit.0 {
            return None;
        }

        let (distance, face_index) = if enter.0 >= t_min && enter.0 <= t_max {
            enter
        } else if exit.0 >= t_min && exit.0 <= t_max {
            exit
        } else {
            return None;
        };

        let local_position = origin + direction * distance;
        let local_normal = (local_position - face_centers[face_index]).normalized();
        let position = ray.at(distance);
        let normal = self
            .orientation
            .local_to_world_vector(local_normal)
            .normalized();
        let uv = Self::uv_from_local_position(local_position);

        if distance.is_finite()
            && vec3_is_finite(position)
            && vec3_is_finite(normal)
            && normal != Vec3::ZERO
            && uv.u.is_finite()
            && uv.v.is_finite()
        {
            Some(Intersection::new(
                distance,
                position,
                normal,
                uv,
                self.material_id,
            ))
        } else {
            None
        }
    }

    fn has_valid_geometry(&self) -> bool {
        vec3_is_finite(self.center)
            && self.circumradius.is_finite()
            && self.circumradius > 0.0
            && self.face_radius_scale.is_finite()
            && self.face_radius_scale >= REULEAUX_FACE_RADIUS_SCALE
    }

    fn local_vertices(self) -> [Vec3; VERTEX_COUNT] {
        let radius = self.circumradius;
        let base_y = -radius / 3.0;
        let base_ring = radius * 8.0_f32.sqrt() / 3.0;
        let (sin_third, cos_third) = (2.0 * std::f32::consts::PI / 3.0).sin_cos();

        [
            Vec3::new(0.0, radius, 0.0),
            Vec3::new(0.0, base_y, base_ring),
            Vec3::new(base_ring * sin_third, base_y, base_ring * cos_third),
            Vec3::new(-base_ring * sin_third, base_y, base_ring * cos_third),
        ]
    }

    /// Centers of the face spheres, in the same order as the opposite
    /// vertices. The face opposite vertex `v` has its circumcenter at `-v / 3`
    /// and its sphere center lies on the line towards `v`, at the distance
    /// that makes the sphere pass through the three face vertices.
    fn local_face_centers(self) -> [Vec3; VERTEX_COUNT] {
        let face_circumradius = self.edge_length() / 3.0_f32.sqrt();
        let radius = self.face_radius();
        let distance_to_face = (radius * radius - face_circumradius * face_circumradius)
            .max(0.0)
            .sqrt();
        let offset = distance_to_face - self.circumradius / 3.0;

        self.local_vertices()
            .map(|vertex| vertex.normalized() * offset)
    }

    fn world_to_local_point(self, point: Vec3) -> Vec3 {
        self.orientation.world_to_local_vector(point - self.center)
    }

    /// Spherical UVs around the center, matching the sphere convention.
    fn uv_from_local_position(local_position: Vec3) -> Vec2 {
        Sphere::uv_from_normal(local_position)
    }
}

fn vec3_is_finite(vector: Vec3) -> bool {
    vector.x.is_finite() && vector.y.is_finite() && vector.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::{CurvedTetrahedron, CurvedTetrahedronError, REULEAUX_FACE_RADIUS_SCALE};
    use crate::{basis::Basis3, math::Vec3, ray::Ray};

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.001, "{left} != {right}");
    }

    fn unit_tetrahedron() -> CurvedTetrahedron {
        CurvedTetrahedron::new(
            Vec3::ZERO,
            1.0,
            REULEAUX_FACE_RADIUS_SCALE,
            Basis3::identity(),
            3,
        )
        .unwrap()
    }

    #[test]
    fn rejects_invalid_center_and_circumradius() {
        assert_eq!(
            CurvedTetrahedron::new(
                Vec3::new(f32::NAN, 0.0, 0.0),
                1.0,
                1.0,
                Basis3::identity(),
                0
            ),
            Err(CurvedTetrahedronError::NonFiniteCenter)
        );
        assert_eq!(
            CurvedTetrahedron::new(Vec3::ZERO, 0.0, 1.0, Basis3::identity(), 0),
            Err(CurvedTetrahedronError::InvalidCircumradius)
        );
        assert_eq!(
            CurvedTetrahedron::new(Vec3::ZERO, f32::INFINITY, 1.0, Basis3::identity(), 0),
            Err(CurvedTetrahedronError::InvalidCircumradius)
        );
        assert_eq!(
            CurvedTetrahedron::new(Vec3::ZERO, 1.0, 0.9, Basis3::identity(), 0),
            Err(CurvedTetrahedronError::InvalidFaceRadiusScale)
        );
        assert_eq!(
            CurvedTetrahedron::new(Vec3::ZERO, 1.0, f32::NAN, Basis3::identity(), 0),
            Err(CurvedTetrahedronError::InvalidFaceRadiusScale)
        );
    }

    #[test]
    fn vertices_form_a_regular_tetrahedron() {
        let tetrahedron = unit_tetrahedron();
        let vertices = tetrahedron.vertices();

        assert!(vertices[0].approx_eq(Vec3::new(0.0, 1.0, 0.0)));
        for vertex in vertices {
            assert_near(vertex.length(), 1.0);
        }
        for first in 0..vertices.len() {
            for second in first + 1..vertices.len() {
                assert_near(
                    (vertices[first] - vertices[second]).length(),
                    tetrahedron.edge_length(),
                );
            }
        }
    }

    #[test]
    fn ray_along_axis_hits_the_apex() {
        let tetrahedron = unit_tetrahedron();
        let ray = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
        let hit = tetrahedron.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 4.0);
        assert!(hit.position.approx_eq(Vec3::new(0.0, 1.0, 0.0)));
        assert!(hit.normal.y > 0.0);
        assert_eq!(hit.material_id, 3);
    }

    #[test]
    fn bottom_face_bulges_below_the_flat_base() {
        let tetrahedron = unit_tetrahedron();
        let ray = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let hit = tetrahedron.intersect(&ray, 0.001, 100.0).unwrap();
        let bulge_bottom = 1.0 - tetrahedron.edge_length();

        assert_near(hit.position.y, bulge_bottom);
        assert!(hit.position.y < -1.0 / 3.0);
        assert!(hit.normal.approx_eq(Vec3::new(0.0, -1.0, 0.0)));
    }

    #[test]
    fn larger_face_radius_flattens_faces_but_keeps_vertices() {
        let flatter = CurvedTetrahedron::new(Vec3::ZERO, 1.0, 1.8, Basis3::identity(), 0).unwrap();
        let reuleaux = unit_tetrahedron();
        let up = Ray::new(Vec3::new(0.0, -5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
        let flatter_bottom = flatter.intersect(&up, 0.001, 100.0).unwrap().position.y;
        let reuleaux_bottom = reuleaux.intersect(&up, 0.001, 100.0).unwrap().position.y;

        assert!(flatter_bottom > reuleaux_bottom);
        assert!(flatter_bottom < -1.0 / 3.0);
        for vertex in flatter.vertices() {
            assert!(flatter.contains_point(vertex));
            assert!(!flatter.contains_point(vertex * 1.01));
        }
    }

    #[test]
    fn hits_lie_on_the_surface_with_outward_normals() {
        for tetrahedron in [
            unit_tetrahedron(),
            CurvedTetrahedron::new(Vec3::ZERO, 1.0, 1.8, Basis3::identity(), 3).unwrap(),
        ] {
            assert_hits_on_surface(tetrahedron);
        }
    }

    fn assert_hits_on_surface(tetrahedron: CurvedTetrahedron) {
        for direction in [
            Vec3::new(1.0, 0.2, 0.3),
            Vec3::new(-0.4, -0.7, 0.5),
            Vec3::new(0.1, 0.9, -0.6),
            Vec3::new(-0.8, 0.1, -0.2),
        ] {
            let origin = direction.normalized() * 6.0;
            let ray = Ray::new(origin, -direction);
            let hit = tetrahedron.intersect(&ray, 0.001, 100.0).unwrap();

            assert!(tetrahedron.contains_point(hit.position));
            assert!(!tetrahedron.contains_point(hit.position + hit.normal * 0.01));
            assert!(hit.normal.dot(ray.direction) < 0.0);
            assert!((0.0..=1.0).contains(&hit.uv.u));
            assert!((0.0..=1.0).contains(&hit.uv.v));
        }
    }

    #[test]
    fn ray_from_inside_hits_the_exit_face() {
        let tetrahedron = unit_tetrahedron();
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        let hit = tetrahedron.intersect(&ray, 0.001, 100.0).unwrap();

        assert_near(hit.distance, 1.0);
        assert!(hit.normal.dot(ray.direction) > 0.0);
    }

    #[test]
    fn misses_rays_outside_and_behind() {
        let tetrahedron = unit_tetrahedron();
        let beside = Ray::new(Vec3::new(3.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let behind = Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 1.0, 0.0));

        assert!(tetrahedron.intersect(&beside, 0.001, 100.0).is_none());
        assert!(tetrahedron.intersect(&behind, 0.001, 100.0).is_none());
        assert!(
            tetrahedron
                .intersect(
                    &Ray::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, -1.0, 0.0)),
                    0.001,
                    2.0
                )
                .is_none()
        );
    }

    #[test]
    fn orientation_rotates_the_apex() {
        let orientation = Basis3::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let tetrahedron =
            CurvedTetrahedron::new(Vec3::new(1.0, 2.0, 3.0), 0.5, 1.6, orientation, 0).unwrap();
        let ray = Ray::new(Vec3::new(6.0, 2.0, 3.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = tetrahedron.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(tetrahedron.vertices()[0].approx_eq(Vec3::new(1.5, 2.0, 3.0)));
        assert!(hit.position.approx_eq(Vec3::new(1.5, 2.0, 3.0)));
    }
}
