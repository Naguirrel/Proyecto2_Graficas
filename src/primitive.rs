use crate::{
    cube::Cube, intersection::Intersection, oriented_box::OrientedBox, ray::Ray, sphere::Sphere,
};

#[derive(Debug, PartialEq)]
pub enum Primitive {
    Cube(Cube),
    Sphere(Sphere),
    OrientedBox(OrientedBox),
}

impl Primitive {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        match self {
            Self::Cube(cube) => cube.intersect(ray, t_min, t_max),
            Self::Sphere(sphere) => sphere.intersect(ray, t_min, t_max),
            Self::OrientedBox(oriented_box) => oriented_box.intersect(ray, t_min, t_max),
        }
    }

    pub fn material_id(&self) -> usize {
        match self {
            Self::Cube(cube) => cube.material_id,
            Self::Sphere(sphere) => sphere.material_id(),
            Self::OrientedBox(oriented_box) => oriented_box.material_id(),
        }
    }

    pub fn as_cube(&self) -> Option<&Cube> {
        match self {
            Self::Cube(cube) => Some(cube),
            Self::Sphere(_) | Self::OrientedBox(_) => None,
        }
    }

    pub fn as_sphere(&self) -> Option<&Sphere> {
        match self {
            Self::Cube(_) | Self::OrientedBox(_) => None,
            Self::Sphere(sphere) => Some(sphere),
        }
    }

    pub fn as_oriented_box(&self) -> Option<&OrientedBox> {
        match self {
            Self::Cube(_) | Self::Sphere(_) => None,
            Self::OrientedBox(oriented_box) => Some(oriented_box),
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

impl From<OrientedBox> for Primitive {
    fn from(oriented_box: OrientedBox) -> Self {
        Self::OrientedBox(oriented_box)
    }
}

#[cfg(test)]
mod tests {
    use super::Primitive;
    use crate::{
        basis::Basis3, cube::Cube, math::Vec3, oriented_box::OrientedBox, ray::Ray, sphere::Sphere,
    };

    fn unit_cube() -> Cube {
        Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 2)
    }

    fn unit_sphere() -> Sphere {
        Sphere::new(Vec3::ZERO, 1.0, 3).unwrap()
    }

    fn unit_oriented_box() -> OrientedBox {
        OrientedBox::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0), Basis3::identity(), 4).unwrap()
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
    fn from_oriented_box_creates_oriented_box_variant() {
        let oriented_box = unit_oriented_box();
        let primitive = Primitive::from(oriented_box);

        assert_eq!(primitive, Primitive::OrientedBox(oriented_box));
    }

    #[test]
    fn as_cube_only_returns_cube() {
        let cube = unit_cube();
        let cube_primitive = Primitive::from(cube);
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(unit_oriented_box());

        assert_eq!(cube_primitive.as_cube(), Some(&cube));
        assert!(sphere_primitive.as_cube().is_none());
        assert!(oriented_box_primitive.as_cube().is_none());
    }

    #[test]
    fn as_sphere_only_returns_sphere() {
        let sphere = unit_sphere();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(sphere);
        let oriented_box_primitive = Primitive::from(unit_oriented_box());

        assert!(cube_primitive.as_sphere().is_none());
        assert_eq!(sphere_primitive.as_sphere(), Some(&sphere));
        assert!(oriented_box_primitive.as_sphere().is_none());
    }

    #[test]
    fn as_oriented_box_only_returns_oriented_box() {
        let oriented_box = unit_oriented_box();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(oriented_box);

        assert!(cube_primitive.as_oriented_box().is_none());
        assert!(sphere_primitive.as_oriented_box().is_none());
        assert_eq!(
            oriented_box_primitive.as_oriented_box(),
            Some(&oriented_box)
        );
    }

    #[test]
    fn material_id_delegates_to_variant() {
        assert_eq!(Primitive::from(unit_cube()).material_id(), 2);
        assert_eq!(Primitive::from(unit_sphere()).material_id(), 3);
        assert_eq!(Primitive::from(unit_oriented_box()).material_id(), 4);
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

    #[test]
    fn oriented_box_intersection_is_delegated() {
        let oriented_box = unit_oriented_box();
        let primitive = Primitive::from(oriented_box);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            oriented_box.intersect(&ray, 0.001, 100.0)
        );
    }
}
