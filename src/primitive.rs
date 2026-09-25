use crate::{cube::Cube, intersection::Intersection, ray::Ray, sphere::Sphere};

#[derive(Debug, PartialEq)]
pub enum Primitive {
    Cube(Cube),
    Sphere(Sphere),
}

impl Primitive {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        match self {
            Self::Cube(cube) => cube.intersect(ray, t_min, t_max),
            Self::Sphere(sphere) => sphere.intersect(ray, t_min, t_max),
        }
    }

    pub fn material_id(&self) -> usize {
        match self {
            Self::Cube(cube) => cube.material_id,
            Self::Sphere(sphere) => sphere.material_id(),
        }
    }

    pub fn as_cube(&self) -> Option<&Cube> {
        match self {
            Self::Cube(cube) => Some(cube),
            Self::Sphere(_) => None,
        }
    }

    pub fn as_sphere(&self) -> Option<&Sphere> {
        match self {
            Self::Cube(_) => None,
            Self::Sphere(sphere) => Some(sphere),
        }
    }
}

impl From<Cube> for Primitive {
    fn from(cube: Cube) -> Self {
        Self::Cube(cube)
    }
}

impl From<Sphere> for Primitive {
    fn from(sphere: Sphere) -> Self {
        Self::Sphere(sphere)
    }
}

#[cfg(test)]
mod tests {
    use super::Primitive;
    use crate::{cube::Cube, math::Vec3, ray::Ray, sphere::Sphere};

    fn unit_cube() -> Cube {
        Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 2)
    }

    fn unit_sphere() -> Sphere {
        Sphere::new(Vec3::ZERO, 1.0, 3).unwrap()
    }

    #[test]
    fn from_cube_creates_cube_variant() {
        let cube = unit_cube();
        let primitive = Primitive::from(cube);

        assert_eq!(primitive, Primitive::Cube(cube));
    }

    #[test]
    fn from_sphere_creates_sphere_variant() {
        let sphere = unit_sphere();
        let primitive = Primitive::from(sphere);

        assert_eq!(primitive, Primitive::Sphere(sphere));
    }

    #[test]
    fn as_cube_only_returns_cube() {
        let cube = unit_cube();
        let cube_primitive = Primitive::from(cube);
        let sphere_primitive = Primitive::from(unit_sphere());

        assert_eq!(cube_primitive.as_cube(), Some(&cube));
        assert!(sphere_primitive.as_cube().is_none());
    }

    #[test]
    fn as_sphere_only_returns_sphere() {
        let sphere = unit_sphere();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(sphere);

        assert!(cube_primitive.as_sphere().is_none());
        assert_eq!(sphere_primitive.as_sphere(), Some(&sphere));
    }

    #[test]
    fn material_id_delegates_to_variant() {
        assert_eq!(Primitive::from(unit_cube()).material_id(), 2);
        assert_eq!(Primitive::from(unit_sphere()).material_id(), 3);
    }

    #[test]
    fn cube_intersection_is_delegated() {
        let cube = unit_cube();
        let primitive = Primitive::from(cube);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            cube.intersect(&ray, 0.001, 100.0)
        );
    }

    #[test]
    fn sphere_intersection_is_delegated() {
        let sphere = unit_sphere();
        let primitive = Primitive::from(sphere);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            sphere.intersect(&ray, 0.001, 100.0)
        );
    }
}
