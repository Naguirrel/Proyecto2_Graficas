use crate::{
    cone::Cone, cube::Cube, curved_tetrahedron::CurvedTetrahedron, cylinder::Cylinder,
    intersection::Intersection, oriented_box::OrientedBox, ray::Ray, sphere::Sphere,
};

#[derive(Debug, PartialEq)]
pub enum Primitive {
    Cube(Cube),
    Sphere(Sphere),
    OrientedBox(OrientedBox),
    Cylinder(Cylinder),
    Cone(Cone),
    CurvedTetrahedron(CurvedTetrahedron),
}

impl Primitive {
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        match self {
            Self::Cube(cube) => cube.intersect(ray, t_min, t_max),
            Self::Sphere(sphere) => sphere.intersect(ray, t_min, t_max),
            Self::OrientedBox(oriented_box) => oriented_box.intersect(ray, t_min, t_max),
            Self::Cylinder(cylinder) => cylinder.intersect(ray, t_min, t_max),
            Self::Cone(cone) => cone.intersect(ray, t_min, t_max),
            Self::CurvedTetrahedron(tetrahedron) => tetrahedron.intersect(ray, t_min, t_max),
        }
    }

    pub fn material_id(&self) -> usize {
        match self {
            Self::Cube(cube) => cube.material_id,
            Self::Sphere(sphere) => sphere.material_id(),
            Self::OrientedBox(oriented_box) => oriented_box.material_id(),
            Self::Cylinder(cylinder) => cylinder.material_id(),
            Self::Cone(cone) => cone.material_id(),
            Self::CurvedTetrahedron(tetrahedron) => tetrahedron.material_id(),
        }
    }

    pub fn as_cube(&self) -> Option<&Cube> {
        match self {
            Self::Cube(cube) => Some(cube),
            Self::Sphere(_)
            | Self::OrientedBox(_)
            | Self::Cylinder(_)
            | Self::Cone(_)
            | Self::CurvedTetrahedron(_) => None,
        }
    }

    pub fn as_sphere(&self) -> Option<&Sphere> {
        match self {
            Self::Cube(_)
            | Self::OrientedBox(_)
            | Self::Cylinder(_)
            | Self::Cone(_)
            | Self::CurvedTetrahedron(_) => None,
            Self::Sphere(sphere) => Some(sphere),
        }
    }

    pub fn as_oriented_box(&self) -> Option<&OrientedBox> {
        match self {
            Self::Cube(_)
            | Self::Sphere(_)
            | Self::Cylinder(_)
            | Self::Cone(_)
            | Self::CurvedTetrahedron(_) => None,
            Self::OrientedBox(oriented_box) => Some(oriented_box),
        }
    }

    pub fn as_cylinder(&self) -> Option<&Cylinder> {
        match self {
            Self::Cube(_)
            | Self::Sphere(_)
            | Self::OrientedBox(_)
            | Self::Cone(_)
            | Self::CurvedTetrahedron(_) => None,
            Self::Cylinder(cylinder) => Some(cylinder),
        }
    }

    pub fn as_cone(&self) -> Option<&Cone> {
        match self {
            Self::Cube(_)
            | Self::Sphere(_)
            | Self::OrientedBox(_)
            | Self::Cylinder(_)
            | Self::CurvedTetrahedron(_) => None,
            Self::Cone(cone) => Some(cone),
        }
    }

    pub fn as_curved_tetrahedron(&self) -> Option<&CurvedTetrahedron> {
        match self {
            Self::CurvedTetrahedron(tetrahedron) => Some(tetrahedron),
            Self::Cube(_)
            | Self::Sphere(_)
            | Self::OrientedBox(_)
            | Self::Cylinder(_)
            | Self::Cone(_) => None,
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

impl From<Cylinder> for Primitive {
    fn from(cylinder: Cylinder) -> Self {
        Self::Cylinder(cylinder)
    }
}

impl From<Cone> for Primitive {
    fn from(cone: Cone) -> Self {
        Self::Cone(cone)
    }
}

impl From<CurvedTetrahedron> for Primitive {
    fn from(tetrahedron: CurvedTetrahedron) -> Self {
        Self::CurvedTetrahedron(tetrahedron)
    }
}

#[cfg(test)]
mod tests {
    use super::Primitive;
    use crate::{
        basis::Basis3, cone::Cone, cube::Cube, curved_tetrahedron::CurvedTetrahedron,
        cylinder::Cylinder, math::Vec3, oriented_box::OrientedBox, ray::Ray, sphere::Sphere,
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

    fn unit_cylinder() -> Cylinder {
        Cylinder::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), 5).unwrap()
    }

    fn unit_cone() -> Cone {
        Cone::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), 6).unwrap()
    }

    fn unit_curved_tetrahedron() -> CurvedTetrahedron {
        CurvedTetrahedron::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), 7).unwrap()
    }

    #[test]
    fn from_curved_tetrahedron_creates_curved_tetrahedron_variant() {
        let tetrahedron = unit_curved_tetrahedron();
        let primitive = Primitive::from(tetrahedron);

        assert_eq!(primitive, Primitive::CurvedTetrahedron(tetrahedron));
    }

    #[test]
    fn as_curved_tetrahedron_only_returns_curved_tetrahedron() {
        let tetrahedron = unit_curved_tetrahedron();
        let tetrahedron_primitive = Primitive::from(tetrahedron);

        assert_eq!(
            tetrahedron_primitive.as_curved_tetrahedron(),
            Some(&tetrahedron)
        );
        assert!(tetrahedron_primitive.as_cube().is_none());
        assert!(tetrahedron_primitive.as_sphere().is_none());
        assert!(tetrahedron_primitive.as_oriented_box().is_none());
        assert!(tetrahedron_primitive.as_cylinder().is_none());
        assert!(tetrahedron_primitive.as_cone().is_none());
        assert!(
            Primitive::from(unit_cube())
                .as_curved_tetrahedron()
                .is_none()
        );
        assert!(
            Primitive::from(unit_sphere())
                .as_curved_tetrahedron()
                .is_none()
        );
        assert!(
            Primitive::from(unit_oriented_box())
                .as_curved_tetrahedron()
                .is_none()
        );
        assert!(
            Primitive::from(unit_cylinder())
                .as_curved_tetrahedron()
                .is_none()
        );
        assert!(
            Primitive::from(unit_cone())
                .as_curved_tetrahedron()
                .is_none()
        );
    }

    #[test]
    fn curved_tetrahedron_intersection_and_material_are_delegated() {
        let tetrahedron = unit_curved_tetrahedron();
        let primitive = Primitive::from(tetrahedron);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(primitive.material_id(), 7);
        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            tetrahedron.intersect(&ray, 0.001, 100.0)
        );
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
    fn from_cylinder_creates_cylinder_variant() {
        let cylinder = unit_cylinder();
        let primitive = Primitive::from(cylinder);

        assert_eq!(primitive, Primitive::Cylinder(cylinder));
    }

    #[test]
    fn from_cone_creates_cone_variant() {
        let cone = unit_cone();
        let primitive = Primitive::from(cone);

        assert_eq!(primitive, Primitive::Cone(cone));
    }

    #[test]
    fn as_cube_only_returns_cube() {
        let cube = unit_cube();
        let cube_primitive = Primitive::from(cube);
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(unit_oriented_box());
        let cylinder_primitive = Primitive::from(unit_cylinder());
        let cone_primitive = Primitive::from(unit_cone());

        assert_eq!(cube_primitive.as_cube(), Some(&cube));
        assert!(sphere_primitive.as_cube().is_none());
        assert!(oriented_box_primitive.as_cube().is_none());
        assert!(cylinder_primitive.as_cube().is_none());
        assert!(cone_primitive.as_cube().is_none());
    }

    #[test]
    fn as_sphere_only_returns_sphere() {
        let sphere = unit_sphere();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(sphere);
        let oriented_box_primitive = Primitive::from(unit_oriented_box());
        let cylinder_primitive = Primitive::from(unit_cylinder());
        let cone_primitive = Primitive::from(unit_cone());

        assert!(cube_primitive.as_sphere().is_none());
        assert_eq!(sphere_primitive.as_sphere(), Some(&sphere));
        assert!(oriented_box_primitive.as_sphere().is_none());
        assert!(cylinder_primitive.as_sphere().is_none());
        assert!(cone_primitive.as_sphere().is_none());
    }

    #[test]
    fn as_oriented_box_only_returns_oriented_box() {
        let oriented_box = unit_oriented_box();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(oriented_box);
        let cylinder_primitive = Primitive::from(unit_cylinder());
        let cone_primitive = Primitive::from(unit_cone());

        assert!(cube_primitive.as_oriented_box().is_none());
        assert!(sphere_primitive.as_oriented_box().is_none());
        assert!(cylinder_primitive.as_oriented_box().is_none());
        assert!(cone_primitive.as_oriented_box().is_none());
        assert_eq!(
            oriented_box_primitive.as_oriented_box(),
            Some(&oriented_box)
        );
    }

    #[test]
    fn as_cylinder_only_returns_cylinder() {
        let cylinder = unit_cylinder();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(unit_oriented_box());
        let cylinder_primitive = Primitive::from(cylinder);
        let cone_primitive = Primitive::from(unit_cone());

        assert!(cube_primitive.as_cylinder().is_none());
        assert!(sphere_primitive.as_cylinder().is_none());
        assert!(oriented_box_primitive.as_cylinder().is_none());
        assert_eq!(cylinder_primitive.as_cylinder(), Some(&cylinder));
        assert!(cone_primitive.as_cylinder().is_none());
    }

    #[test]
    fn as_cone_only_returns_cone() {
        let cone = unit_cone();
        let cube_primitive = Primitive::from(unit_cube());
        let sphere_primitive = Primitive::from(unit_sphere());
        let oriented_box_primitive = Primitive::from(unit_oriented_box());
        let cylinder_primitive = Primitive::from(unit_cylinder());
        let cone_primitive = Primitive::from(cone);

        assert!(cube_primitive.as_cone().is_none());
        assert!(sphere_primitive.as_cone().is_none());
        assert!(oriented_box_primitive.as_cone().is_none());
        assert!(cylinder_primitive.as_cone().is_none());
        assert_eq!(cone_primitive.as_cone(), Some(&cone));
    }

    #[test]
    fn material_id_delegates_to_variant() {
        assert_eq!(Primitive::from(unit_cube()).material_id(), 2);
        assert_eq!(Primitive::from(unit_sphere()).material_id(), 3);
        assert_eq!(Primitive::from(unit_oriented_box()).material_id(), 4);
        assert_eq!(Primitive::from(unit_cylinder()).material_id(), 5);
        assert_eq!(Primitive::from(unit_cone()).material_id(), 6);
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

    #[test]
    fn cylinder_intersection_is_delegated() {
        let cylinder = unit_cylinder();
        let primitive = Primitive::from(cylinder);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            cylinder.intersect(&ray, 0.001, 100.0)
        );
    }

    #[test]
    fn cone_intersection_is_delegated() {
        let cone = unit_cone();
        let primitive = Primitive::from(cone);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            primitive.intersect(&ray, 0.001, 100.0),
            cone.intersect(&ray, 0.001, 100.0)
        );
    }
}
