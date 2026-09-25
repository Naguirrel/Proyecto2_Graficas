use crate::{
    color::Color, cube::Cube, cylinder::Cylinder, intersection::Intersection, light::PointLight,
    material::Material, oriented_box::OrientedBox, primitive::Primitive, ray::Ray, skybox::Skybox,
    sphere::Sphere, texture::Texture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    MissingMaterial { material_id: usize },
    MissingTexture { texture_id: usize },
}

#[derive(Debug)]
pub struct Scene {
    objects: Vec<Primitive>,
    materials: Vec<Material>,
    textures: Vec<Texture>,
    lights: Vec<PointLight>,
    ambient_light: Color,
    skybox: Option<Skybox>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_texture(&mut self, texture: Texture) -> usize {
        let texture_id = self.textures.len();
        self.textures.push(texture);
        texture_id
    }

    pub fn add_material(&mut self, material: Material) -> Result<usize, SceneError> {
        if let Some(texture_id) = material.texture_id
            && self.texture(texture_id).is_none()
        {
            return Err(SceneError::MissingTexture { texture_id });
        }

        let material_id = self.materials.len();
        self.materials.push(material);
        Ok(material_id)
    }

    pub fn add_cube(&mut self, cube: Cube) -> Result<(), SceneError> {
        self.add_primitive(cube.into())
    }

    pub fn add_sphere(&mut self, sphere: Sphere) -> Result<(), SceneError> {
        self.add_primitive(sphere.into())
    }

    pub fn add_oriented_box(&mut self, oriented_box: OrientedBox) -> Result<(), SceneError> {
        self.add_primitive(oriented_box.into())
    }

    pub fn add_cylinder(&mut self, cylinder: Cylinder) -> Result<(), SceneError> {
        self.add_primitive(cylinder.into())
    }

    pub fn add_primitive(&mut self, primitive: Primitive) -> Result<(), SceneError> {
        let material_id = primitive.material_id();

        if self.material(material_id).is_none() {
            return Err(SceneError::MissingMaterial { material_id });
        }

        self.objects.push(primitive);
        Ok(())
    }

    pub fn add_light(&mut self, light: PointLight) {
        self.lights.push(light);
    }

    pub fn set_skybox(&mut self, skybox: Skybox) {
        self.skybox = Some(skybox);
    }

    pub fn skybox(&self) -> Option<&Skybox> {
        self.skybox.as_ref()
    }

    pub fn material(&self, material_id: usize) -> Option<&Material> {
        self.materials.get(material_id)
    }

    pub fn texture(&self, texture_id: usize) -> Option<&Texture> {
        self.textures.get(texture_id)
    }

    pub fn objects(&self) -> &[Primitive] {
        &self.objects
    }

    /// Iterates over cube primitives only, without allocating a temporary list.
    pub fn cubes(&self) -> impl Iterator<Item = &Cube> {
        self.objects.iter().filter_map(Primitive::as_cube)
    }

    /// Iterates over oriented-box primitives only, without allocating.
    pub fn oriented_boxes(&self) -> impl Iterator<Item = &OrientedBox> {
        self.objects.iter().filter_map(Primitive::as_oriented_box)
    }

    /// Iterates over cylinder primitives only, without allocating.
    pub fn cylinders(&self) -> impl Iterator<Item = &Cylinder> {
        self.objects.iter().filter_map(Primitive::as_cylinder)
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn cube_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|primitive| primitive.as_cube().is_some())
            .count()
    }

    pub fn sphere_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|primitive| primitive.as_sphere().is_some())
            .count()
    }

    pub fn oriented_box_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|primitive| primitive.as_oriented_box().is_some())
            .count()
    }

    pub fn cylinder_count(&self) -> usize {
        self.objects
            .iter()
            .filter(|primitive| primitive.as_cylinder().is_some())
            .count()
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    pub fn textures(&self) -> &[Texture] {
        &self.textures
    }

    pub fn lights(&self) -> &[PointLight] {
        &self.lights
    }

    pub fn ambient_light(&self) -> Color {
        self.ambient_light
    }

    pub fn set_ambient_light(&mut self, ambient_light: Color) {
        self.ambient_light = ambient_light.clamped();
    }

    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Intersection> {
        let mut closest = t_max;
        let mut closest_hit = None;

        for primitive in &self.objects {
            if let Some(hit) = primitive.intersect(ray, t_min, closest) {
                closest = hit.distance;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            materials: Vec::new(),
            textures: Vec::new(),
            lights: Vec::new(),
            ambient_light: Color::new(0.08, 0.08, 0.08),
            skybox: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Scene, SceneError};
    use crate::{
        basis::Basis3,
        color::Color,
        cube::Cube,
        cylinder::Cylinder,
        light::PointLight,
        material::Material,
        math::{Vec2, Vec3},
        oriented_box::OrientedBox,
        primitive::Primitive,
        ray::Ray,
        skybox::Skybox,
        sphere::Sphere,
        texture::{Texture, WrapMode},
    };

    fn diffuse_scene() -> Scene {
        let mut scene = Scene::new();
        scene.add_material(Material::diffuse(Color::WHITE)).unwrap();
        scene
    }

    fn white_texture() -> Texture {
        Texture::new(1, 1, vec![Color::WHITE]).unwrap()
    }

    fn unit_cube(material_id: usize) -> Cube {
        Cube::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            material_id,
        )
    }

    fn unit_sphere(material_id: usize) -> Sphere {
        Sphere::new(Vec3::ZERO, 1.0, material_id).unwrap()
    }

    fn unit_oriented_box(material_id: usize) -> OrientedBox {
        OrientedBox::new(
            Vec3::ZERO,
            Vec3::new(1.0, 1.0, 1.0),
            Basis3::identity(),
            material_id,
        )
        .unwrap()
    }

    fn unit_cylinder(material_id: usize) -> Cylinder {
        Cylinder::new(Vec3::ZERO, 1.0, 1.0, Basis3::identity(), material_id).unwrap()
    }

    #[test]
    fn new_scene_starts_empty() {
        let scene = Scene::new();

        assert!(scene.objects().is_empty());
        assert_eq!(scene.object_count(), 0);
        assert_eq!(scene.cube_count(), 0);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 0);
        assert!(scene.materials().is_empty());
        assert!(scene.textures().is_empty());
        assert!(scene.lights().is_empty());
        assert!(scene.skybox().is_none());
    }

    #[test]
    fn default_scene_matches_new_scene_state() {
        let new_scene = Scene::new();
        let default_scene = Scene::default();

        assert_eq!(default_scene.objects(), new_scene.objects());
        assert_eq!(default_scene.object_count(), new_scene.object_count());
        assert_eq!(default_scene.cube_count(), new_scene.cube_count());
        assert_eq!(default_scene.sphere_count(), new_scene.sphere_count());
        assert_eq!(
            default_scene.oriented_box_count(),
            new_scene.oriented_box_count()
        );
        assert_eq!(default_scene.cylinder_count(), new_scene.cylinder_count());
        assert_eq!(default_scene.materials(), new_scene.materials());
        assert_eq!(default_scene.textures(), new_scene.textures());
        assert_eq!(default_scene.lights(), new_scene.lights());
        assert_eq!(default_scene.ambient_light(), new_scene.ambient_light());
        assert_eq!(default_scene.skybox(), new_scene.skybox());
    }

    #[test]
    fn adding_material_returns_correct_index() {
        let mut scene = Scene::new();

        assert_eq!(scene.add_material(Material::default()), Ok(0));
        assert_eq!(scene.add_material(Material::default()), Ok(1));
    }

    #[test]
    fn valid_material_lookup_succeeds() {
        let mut scene = Scene::new();
        let material_id = scene
            .add_material(Material::diffuse(Color::new(0.2, 0.3, 0.4)))
            .unwrap();

        assert_eq!(
            scene.material(material_id).unwrap().albedo,
            Color::new(0.2, 0.3, 0.4)
        );
    }

    #[test]
    fn invalid_material_lookup_returns_none() {
        assert!(Scene::new().material(20).is_none());
    }

    #[test]
    fn adding_texture_returns_correct_index() {
        let mut scene = Scene::new();

        assert_eq!(scene.add_texture(white_texture()), 0);
        assert_eq!(scene.add_texture(white_texture()), 1);
    }

    #[test]
    fn valid_texture_lookup_succeeds() {
        let mut scene = Scene::new();
        let texture_id = scene.add_texture(white_texture());

        assert_eq!(scene.texture(texture_id).unwrap().width(), 1);
        assert_eq!(scene.texture(texture_id).unwrap().height(), 1);
    }

    #[test]
    fn invalid_texture_lookup_returns_none() {
        assert!(Scene::new().texture(20).is_none());
    }

    #[test]
    fn material_with_missing_texture_is_rejected() {
        let mut scene = Scene::new();
        let material =
            Material::diffuse(Color::WHITE).with_texture(4, Vec2::new(1.0, 1.0), WrapMode::Repeat);

        assert_eq!(
            scene.add_material(material),
            Err(SceneError::MissingTexture { texture_id: 4 })
        );
        assert!(scene.materials().is_empty());
    }

    #[test]
    fn material_with_existing_texture_is_registered() {
        let mut scene = Scene::new();
        let texture_id = scene.add_texture(white_texture());
        let material = Material::diffuse(Color::WHITE).with_texture(
            texture_id,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        );

        assert_eq!(scene.add_material(material), Ok(0));
    }

    #[test]
    fn new_scene_has_no_lights() {
        assert!(Scene::new().lights().is_empty());
    }

    #[test]
    fn adding_lights_preserves_order() {
        let mut scene = Scene::new();
        let first = PointLight::new(Vec3::new(1.0, 0.0, 0.0), Color::WHITE, 1.0);
        let second = PointLight::new(Vec3::new(2.0, 0.0, 0.0), Color::new(0.5, 0.5, 1.0), 2.0);

        scene.add_light(first);
        scene.add_light(second);

        assert_eq!(scene.lights(), &[first, second]);
    }

    #[test]
    fn skybox_can_be_configured_and_read() {
        let mut scene = Scene::new();
        let skybox = Skybox::new(white_texture()).with_intensity(0.5);

        scene.set_skybox(skybox);

        assert!(scene.skybox().is_some());
        assert_eq!(scene.skybox().unwrap().texture().width(), 1);
        assert_eq!(scene.skybox().unwrap().intensity(), 0.5);
    }

    #[test]
    fn ambient_light_can_be_configured_and_read() {
        let mut scene = Scene::new();

        scene.set_ambient_light(Color::new(0.2, 0.1, 0.05));

        assert_eq!(scene.ambient_light(), Color::new(0.2, 0.1, 0.05));
    }

    #[test]
    fn invalid_ambient_values_are_clamped_safely() {
        let mut scene = Scene::new();

        scene.set_ambient_light(Color::new(-1.0, f32::INFINITY, 2.0));

        assert_eq!(scene.ambient_light(), Color::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn adding_cube_with_valid_material_succeeds() {
        let mut scene = diffuse_scene();
        let cube = unit_cube(0);

        assert_eq!(scene.add_cube(cube), Ok(()));
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.cube_count(), 1);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 0);
        assert_eq!(scene.objects()[0].as_cube(), Some(&cube));
        assert_eq!(scene.cubes().count(), 1);
    }

    #[test]
    fn adding_sphere_with_valid_material_succeeds() {
        let mut scene = diffuse_scene();
        let sphere = unit_sphere(0);

        assert_eq!(scene.add_sphere(sphere), Ok(()));
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.cube_count(), 0);
        assert_eq!(scene.sphere_count(), 1);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 0);
        assert_eq!(scene.objects()[0].as_sphere(), Some(&sphere));
    }

    #[test]
    fn adding_oriented_box_with_valid_material_succeeds() {
        let mut scene = diffuse_scene();
        let oriented_box = unit_oriented_box(0);

        assert_eq!(scene.add_oriented_box(oriented_box), Ok(()));
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.cube_count(), 0);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 1);
        assert_eq!(scene.cylinder_count(), 0);
        assert_eq!(scene.objects()[0].as_oriented_box(), Some(&oriented_box));
        assert_eq!(scene.oriented_boxes().count(), 1);
    }

    #[test]
    fn adding_cylinder_with_valid_material_succeeds() {
        let mut scene = diffuse_scene();
        let cylinder = unit_cylinder(0);

        assert_eq!(scene.add_cylinder(cylinder), Ok(()));
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.cube_count(), 0);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 1);
        assert_eq!(scene.objects()[0].as_cylinder(), Some(&cylinder));
        assert_eq!(scene.cylinders().count(), 1);
    }

    #[test]
    fn add_primitive_accepts_cube_sphere_oriented_box_and_cylinder() {
        let mut scene = diffuse_scene();

        assert_eq!(scene.add_primitive(Primitive::from(unit_cube(0))), Ok(()));
        assert_eq!(scene.add_primitive(Primitive::from(unit_sphere(0))), Ok(()));
        assert_eq!(
            scene.add_primitive(Primitive::from(unit_oriented_box(0))),
            Ok(())
        );
        assert_eq!(
            scene.add_primitive(Primitive::from(unit_cylinder(0))),
            Ok(())
        );
        assert_eq!(scene.object_count(), 4);
        assert_eq!(scene.cube_count(), 1);
        assert_eq!(scene.sphere_count(), 1);
        assert_eq!(scene.oriented_box_count(), 1);
        assert_eq!(scene.cylinder_count(), 1);
    }

    #[test]
    fn adding_cube_with_invalid_material_returns_error() {
        let mut scene = Scene::new();
        let cube = unit_cube(3);

        assert_eq!(
            scene.add_cube(cube),
            Err(SceneError::MissingMaterial { material_id: 3 })
        );
        assert_eq!(scene.object_count(), 0);
    }

    #[test]
    fn adding_sphere_with_invalid_material_returns_error() {
        let mut scene = Scene::new();
        let sphere = unit_sphere(3);

        assert_eq!(
            scene.add_sphere(sphere),
            Err(SceneError::MissingMaterial { material_id: 3 })
        );
        assert_eq!(scene.object_count(), 0);
    }

    #[test]
    fn adding_oriented_box_with_invalid_material_returns_error() {
        let mut scene = Scene::new();
        let oriented_box = unit_oriented_box(3);

        assert_eq!(
            scene.add_oriented_box(oriented_box),
            Err(SceneError::MissingMaterial { material_id: 3 })
        );
        assert_eq!(scene.object_count(), 0);
    }

    #[test]
    fn adding_cylinder_with_invalid_material_returns_error() {
        let mut scene = Scene::new();
        let cylinder = unit_cylinder(3);

        assert_eq!(
            scene.add_cylinder(cylinder),
            Err(SceneError::MissingMaterial { material_id: 3 })
        );
        assert_eq!(scene.object_count(), 0);
    }

    #[test]
    fn failed_add_primitive_does_not_modify_scene() {
        let mut scene = diffuse_scene();
        scene.add_cube(unit_cube(0)).unwrap();

        assert_eq!(
            scene.add_primitive(Primitive::from(unit_sphere(9))),
            Err(SceneError::MissingMaterial { material_id: 9 })
        );
        assert_eq!(scene.object_count(), 1);
        assert_eq!(scene.cube_count(), 1);
        assert_eq!(scene.sphere_count(), 0);
        assert_eq!(scene.oriented_box_count(), 0);
        assert_eq!(scene.cylinder_count(), 0);
    }

    #[test]
    fn empty_scene_intersection_returns_none() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(Scene::new().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn scene_intersection_finds_cube() {
        let mut scene = diffuse_scene();
        scene.add_cube(unit_cube(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn scene_intersection_finds_sphere() {
        let mut scene = diffuse_scene();
        scene.add_sphere(unit_sphere(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn scene_intersection_finds_oriented_box() {
        let mut scene = diffuse_scene();
        scene.add_oriented_box(unit_oriented_box(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn scene_intersection_finds_cylinder() {
        let mut scene = diffuse_scene();
        scene.add_cylinder(unit_cylinder(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn mixed_scene_returns_cube_when_cube_is_closest() {
        let mut scene = diffuse_scene();
        let cube_id = scene
            .add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)))
            .unwrap();
        scene
            .add_sphere(Sphere::new(Vec3::new(0.0, 0.0, -3.0), 0.5, 0).unwrap())
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, 1.0),
                Vec3::new(0.5, 0.5, 2.0),
                cube_id,
            ))
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, cube_id);
    }

    #[test]
    fn mixed_scene_returns_sphere_when_sphere_is_closest() {
        let mut scene = diffuse_scene();
        let sphere_id = scene
            .add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, -3.0),
                Vec3::new(0.5, 0.5, -2.0),
                0,
            ))
            .unwrap();
        scene
            .add_sphere(Sphere::new(Vec3::new(0.0, 0.0, 1.5), 0.5, sphere_id).unwrap())
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, sphere_id);
    }

    #[test]
    fn mixed_scene_returns_oriented_box_when_oriented_box_is_closest() {
        let mut scene = diffuse_scene();
        let oriented_box_id = scene
            .add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)))
            .unwrap();
        let orientation =
            Basis3::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2).unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, -3.0),
                Vec3::new(0.5, 0.5, -2.0),
                0,
            ))
            .unwrap();
        scene
            .add_sphere(Sphere::new(Vec3::new(0.0, 0.0, -1.5), 0.5, 0).unwrap())
            .unwrap();
        scene
            .add_oriented_box(
                OrientedBox::new(
                    Vec3::new(0.0, 0.0, 1.5),
                    Vec3::new(0.5, 0.5, 0.25),
                    orientation,
                    oriented_box_id,
                )
                .unwrap(),
            )
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, oriented_box_id);
    }

    #[test]
    fn mixed_scene_returns_cylinder_when_cylinder_is_closest() {
        let mut scene = diffuse_scene();
        let cylinder_id = scene
            .add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, -3.0),
                Vec3::new(0.5, 0.5, -2.0),
                0,
            ))
            .unwrap();
        scene
            .add_sphere(Sphere::new(Vec3::new(0.0, 0.0, -1.5), 0.5, 0).unwrap())
            .unwrap();
        scene
            .add_cylinder(
                Cylinder::new(
                    Vec3::new(0.0, 0.0, 1.5),
                    0.5,
                    0.5,
                    Basis3::identity(),
                    cylinder_id,
                )
                .unwrap(),
            )
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, cylinder_id);
    }

    #[test]
    fn insertion_order_does_not_change_closest_hit() {
        let mut first = diffuse_scene();
        let mut second = diffuse_scene();
        let near = Sphere::new(Vec3::new(0.0, 0.0, 1.5), 0.5, 0).unwrap();
        let far = Cube::new(Vec3::new(-0.5, -0.5, -3.0), Vec3::new(0.5, 0.5, -2.0), 0);
        first.add_cube(far).unwrap();
        first.add_sphere(near).unwrap();
        second.add_sphere(near).unwrap();
        second.add_cube(far).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            first.intersect(&ray, 0.001, 100.0).unwrap().distance,
            second.intersect(&ray, 0.001, 100.0).unwrap().distance
        );
    }

    #[test]
    fn intersection_respects_t_min_and_t_max() {
        let mut scene = diffuse_scene();
        scene.add_sphere(unit_sphere(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 1.0).is_none());
        assert_eq!(scene.intersect(&ray, 2.5, 100.0).unwrap().distance, 4.0);
    }

    #[test]
    fn hit_material_id_matches_cube() {
        let mut scene = Scene::new();
        let red = scene
            .add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)))
            .unwrap();
        let blue = scene
            .add_material(Material::diffuse(Color::new(0.0, 0.0, 1.0)))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, 1.0),
                Vec3::new(1.0, 1.0, 2.0),
                blue,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -2.0),
                Vec3::new(1.0, 1.0, -1.0),
                red,
            ))
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));

        assert_eq!(
            scene.intersect(&ray, 0.001, 100.0).unwrap().material_id,
            blue
        );
    }

    #[test]
    fn scene_intersection_preserves_cube_uv() {
        let mut scene = diffuse_scene();
        scene.add_cube(unit_cube(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.5, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(Vec2::new(0.75, 0.625)));
    }

    #[test]
    fn scene_intersection_preserves_sphere_uv() {
        let mut scene = diffuse_scene();
        scene.add_sphere(unit_sphere(0)).unwrap();
        let ray = Ray::new(Vec3::new(3.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(Vec2::new(0.5, 0.5)));
    }

    #[test]
    fn scene_intersection_preserves_oriented_box_uv() {
        let mut scene = diffuse_scene();
        scene.add_oriented_box(unit_oriented_box(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.5, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(Vec2::new(0.75, 0.625)));
    }

    #[test]
    fn scene_intersection_preserves_cylinder_uv() {
        let mut scene = diffuse_scene();
        scene.add_cylinder(unit_cylinder(0)).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(Vec2::new(0.75, 0.625)));
    }

    #[test]
    fn ray_missing_all_primitives_returns_none() {
        let mut scene = diffuse_scene();
        scene.add_cube(unit_cube(0)).unwrap();
        scene
            .add_sphere(Sphere::new(Vec3::new(4.0, 0.0, 0.0), 1.0, 0).unwrap())
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 1.0, 0.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn counts_do_not_modify_scene() {
        let mut scene = diffuse_scene();
        scene.add_cube(unit_cube(0)).unwrap();
        scene.add_sphere(unit_sphere(0)).unwrap();
        scene.add_oriented_box(unit_oriented_box(0)).unwrap();
        scene.add_cylinder(unit_cylinder(0)).unwrap();
        let objects_before = scene.object_count();

        assert_eq!(scene.cube_count(), 1);
        assert_eq!(scene.sphere_count(), 1);
        assert_eq!(scene.oriented_box_count(), 1);
        assert_eq!(scene.cylinder_count(), 1);
        assert_eq!(scene.object_count(), objects_before);
    }
}
