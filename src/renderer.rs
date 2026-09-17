use crate::{
    camera::Camera, color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection,
    material::Material, math::Vec3, ray::Ray, scene::Scene,
};

const HIT_T_MIN: f32 = 0.001;
const HIT_T_MAX: f32 = 1_000.0;

pub fn render_background(framebuffer: &mut Framebuffer) {
    let aspect_ratio = framebuffer.width() as f32 / framebuffer.height().max(1) as f32;
    let camera = Camera::new(
        Vec3::new(3.0, 2.0, 5.0),
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        55.0,
        aspect_ratio,
    );
    let scene = sample_scene();

    framebuffer.clear(Color::BLACK);

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray = camera.ray_for_pixel(x, y, framebuffer.width(), framebuffer.height());
            let color = trace_primary_ray(&ray, &scene);

            framebuffer.set_pixel(x, y, color);
        }
    }
}

pub(crate) fn sample_scene() -> Scene {
    let mut scene = Scene::new();
    let blue = scene.add_material(Material::diffuse(Color::new(0.28, 0.42, 0.78)));
    let red = scene.add_material(Material::diffuse(Color::new(0.72, 0.22, 0.18)));

    scene
        .add_cube(Cube::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            blue,
        ))
        .expect("sample cube uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(1.25, -1.0, -0.4),
            Vec3::new(1.85, -0.4, 0.2),
            red,
        ))
        .expect("sample accent cube uses a registered material");

    scene
}

pub(crate) fn trace_primary_ray(ray: &Ray, scene: &Scene) -> Color {
    match scene.intersect(ray, HIT_T_MIN, HIT_T_MAX) {
        Some(hit) => hit_color(hit, scene),
        None => background_color(ray.direction),
    }
}

pub(crate) fn hit_color(hit: Intersection, scene: &Scene) -> Color {
    let normal_color = Color::new(
        hit.normal.x * 0.5 + 0.5,
        hit.normal.y * 0.5 + 0.5,
        hit.normal.z * 0.5 + 0.5,
    );
    let material = scene.material(hit.material_id).copied().unwrap_or_default();

    (material.albedo * 0.55 + normal_color * 0.45 + material.emission).clamped()
}

pub(crate) fn background_color(direction: Vec3) -> Color {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let lower = Color::new(0.06, 0.07, 0.10);
    let upper = Color::new(0.42, 0.55, 0.72);

    lower.lerp(upper, t).clamped()
}

#[cfg(test)]
mod tests {
    use super::{background_color, hit_color, render_background, sample_scene, trace_primary_ray};
    use crate::{
        color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection, math::Vec3,
        ray::Ray, scene::Scene,
    };

    fn scene_with_main_cube() -> Scene {
        let mut scene = Scene::new();
        let material_id = scene.add_material(crate::material::Material::diffuse(Color::new(
            0.28, 0.42, 0.78,
        )));
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();
        scene
    }

    #[test]
    fn central_ray_hits_cube() {
        let camera_ray = Ray::new(
            Vec3::new(3.0, 2.0, 5.0),
            Vec3::ZERO - Vec3::new(3.0, 2.0, 5.0),
        );
        let scene = scene_with_main_cube();

        assert!(scene.intersect(&camera_ray, 0.001, 100.0).is_some());
    }

    #[test]
    fn ray_missing_cube_uses_background() {
        let ray = Ray::new(Vec3::new(3.0, 2.0, 5.0), Vec3::new(0.0, 1.0, 0.0));
        let scene = scene_with_main_cube();

        assert_eq!(
            trace_primary_ray(&ray, &scene).to_u32(),
            background_color(ray.direction).to_u32()
        );
    }

    #[test]
    fn two_materials_can_produce_different_colors() {
        let scene = sample_scene();
        let first = Intersection::new(1.0, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), 0);
        let second = Intersection::new(1.0, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), 1);

        assert_ne!(
            hit_color(first, &scene).to_u32(),
            hit_color(second, &scene).to_u32()
        );
    }

    #[test]
    fn scene_intersection_selects_nearest_cube_for_renderer() {
        let mut scene = Scene::new();
        let near_id = scene.add_material(crate::material::Material::diffuse(Color::new(
            0.7, 0.2, 0.2,
        )));
        let far_id = scene.add_material(crate::material::Material::diffuse(Color::new(
            0.2, 0.2, 0.7,
        )));
        let near = Cube::new(
            Vec3::new(-0.5, -0.5, 1.0),
            Vec3::new(0.5, 0.5, 2.0),
            near_id,
        );
        let far = Cube::new(
            Vec3::new(-0.5, -0.5, -2.0),
            Vec3::new(0.5, 0.5, -1.0),
            far_id,
        );
        scene.add_cube(far).unwrap();
        scene.add_cube(near).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, near_id);
    }

    #[test]
    fn hit_color_remains_in_display_range() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let scene = scene_with_main_cube();
        let hit = scene.cubes()[0].intersect(&ray, 0.001, 100.0).unwrap();
        let color = hit_color(hit, &scene);

        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
    }

    #[test]
    fn render_small_framebuffer_contains_cube_and_background_pixels() {
        let mut framebuffer = Framebuffer::new(64, 48);

        render_background(&mut framebuffer);

        let face_colors = [
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ]
        .map(|normal| Intersection::new(1.0, Vec3::ZERO, normal, 0))
        .map(|hit| hit_color(hit, &sample_scene()))
        .map(Color::to_u32);
        let cube_pixels = framebuffer
            .pixels()
            .iter()
            .filter(|pixel| face_colors.contains(pixel))
            .count();
        let background_pixels = framebuffer.pixels().len() - cube_pixels;

        assert!(cube_pixels > 0);
        assert!(background_pixels > 0);
    }

    #[test]
    fn rendered_framebuffer_keeps_size_and_valid_pixels() {
        let mut framebuffer = Framebuffer::new(16, 12);
        let expected_len = framebuffer.pixels().len();

        render_background(&mut framebuffer);

        assert_eq!(framebuffer.pixels().len(), expected_len);
        assert!(
            framebuffer
                .pixels()
                .iter()
                .all(|&pixel| pixel <= 0x00ff_ffff)
        );
    }
}
