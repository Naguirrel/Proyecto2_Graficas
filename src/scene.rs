use crate::{
    color::Color, cube::Cube, intersection::Intersection, light::PointLight, material::Material,
    ray::Ray, skybox::Skybox, texture::Texture,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    MissingMaterial { material_id: usize },
    MissingTexture { texture_id: usize },
}

#[derive(Debug)]
pub struct Scene {
    cubes: Vec<Cube>,
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
        if self.material(cube.material_id).is_none() {
            return Err(SceneError::MissingMaterial {
                material_id: cube.material_id,
            });
        }

        self.cubes.push(cube);
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

    pub fn cubes(&self) -> &[Cube] {
        &self.cubes
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

        for cube in &self.cubes {
            if let Some(hit) = cube.intersect(ray, t_min, closest) {
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
            cubes: Vec::new(),
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
        color::Color,
        cube::Cube,
        light::PointLight,
        material::Material,
        math::{Vec2, Vec3},
        ray::Ray,
        skybox::Skybox,
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

    #[test]
    fn new_scene_starts_empty() {
        let scene = Scene::new();

        assert!(scene.cubes().is_empty());
        assert!(scene.materials().is_empty());
        assert!(scene.textures().is_empty());
        assert!(scene.lights().is_empty());
        assert!(scene.skybox().is_none());
    }

    #[test]
    fn default_scene_matches_new_scene_state() {
        let new_scene = Scene::new();
        let default_scene = Scene::default();

        assert_eq!(default_scene.cubes(), new_scene.cubes());
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
        let cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 0);

        assert_eq!(scene.add_cube(cube), Ok(()));
        assert_eq!(scene.cubes().len(), 1);
    }

    #[test]
    fn adding_cube_with_invalid_material_returns_error() {
        let mut scene = Scene::new();
        let cube = Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 3);

        assert_eq!(
            scene.add_cube(cube),
            Err(SceneError::MissingMaterial { material_id: 3 })
        );
        assert!(scene.cubes().is_empty());
    }

    #[test]
    fn empty_scene_intersection_returns_none() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(Scene::new().intersect(&ray, 0.001, 100.0).is_none());
    }

    #[test]
    fn scene_intersection_finds_cube() {
        let mut scene = diffuse_scene();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                0,
            ))
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));

        assert!(scene.intersect(&ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn scene_intersection_selects_closest_hit() {
        let mut scene = diffuse_scene();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, -2.0),
                Vec3::new(0.5, 0.5, -1.0),
                0,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.5, -0.5, 1.0),
                Vec3::new(0.5, 0.5, 2.0),
                0,
            ))
            .unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!((hit.distance - 2.0).abs() < 0.0001);
    }

    #[test]
    fn insertion_order_does_not_change_closest_hit() {
        let mut first = diffuse_scene();
        let mut second = diffuse_scene();
        let near = Cube::new(Vec3::new(-0.5, -0.5, 1.0), Vec3::new(0.5, 0.5, 2.0), 0);
        let far = Cube::new(Vec3::new(-0.5, -0.5, -2.0), Vec3::new(0.5, 0.5, -1.0), 0);
        first.add_cube(far).unwrap();
        first.add_cube(near).unwrap();
        second.add_cube(near).unwrap();
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
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                0,
            ))
            .unwrap();
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
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                0,
            ))
            .unwrap();
        let ray = Ray::new(Vec3::new(0.5, 0.25, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert!(hit.uv.approx_eq(Vec2::new(0.75, 0.625)));
    }
}
