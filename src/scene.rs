use crate::{cube::Cube, intersection::Intersection, material::Material, ray::Ray};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    MissingMaterial { material_id: usize },
}

#[derive(Debug, Default)]
pub struct Scene {
    cubes: Vec<Cube>,
    materials: Vec<Material>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_material(&mut self, material: Material) -> usize {
        let material_id = self.materials.len();
        self.materials.push(material);
        material_id
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

    pub fn material(&self, material_id: usize) -> Option<&Material> {
        self.materials.get(material_id)
    }

    pub fn cubes(&self) -> &[Cube] {
        &self.cubes
    }

    pub fn materials(&self) -> &[Material] {
        &self.materials
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

#[cfg(test)]
mod tests {
    use super::{Scene, SceneError};
    use crate::{color::Color, cube::Cube, material::Material, math::Vec3, ray::Ray};

    fn diffuse_scene() -> Scene {
        let mut scene = Scene::new();
        scene.add_material(Material::diffuse(Color::WHITE));
        scene
    }

    #[test]
    fn new_scene_starts_empty() {
        let scene = Scene::new();

        assert!(scene.cubes().is_empty());
        assert!(scene.materials().is_empty());
    }

    #[test]
    fn adding_material_returns_correct_index() {
        let mut scene = Scene::new();

        assert_eq!(scene.add_material(Material::default()), 0);
        assert_eq!(scene.add_material(Material::default()), 1);
    }

    #[test]
    fn valid_material_lookup_succeeds() {
        let mut scene = Scene::new();
        let material_id = scene.add_material(Material::diffuse(Color::new(0.2, 0.3, 0.4)));

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
        let red = scene.add_material(Material::diffuse(Color::new(1.0, 0.0, 0.0)));
        let blue = scene.add_material(Material::diffuse(Color::new(0.0, 0.0, 1.0)));
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
}
