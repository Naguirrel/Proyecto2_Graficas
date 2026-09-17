use crate::{
    camera::Camera, color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection,
    math::Vec3, ray::Ray,
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
    let cubes = [Cube::new(
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
        0,
    )];

    framebuffer.clear(Color::BLACK);

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray = camera.ray_for_pixel(x, y, framebuffer.width(), framebuffer.height());
            let color = trace_primary_ray(&ray, &cubes);

            framebuffer.set_pixel(x, y, color);
        }
    }
}

pub(crate) fn trace_primary_ray(ray: &Ray, cubes: &[Cube]) -> Color {
    match closest_hit(ray, cubes, HIT_T_MIN, HIT_T_MAX) {
        Some(hit) => hit_color(hit),
        None => background_color(ray.direction),
    }
}

pub(crate) fn closest_hit(
    ray: &Ray,
    cubes: &[Cube],
    t_min: f32,
    t_max: f32,
) -> Option<Intersection> {
    let mut closest = t_max;
    let mut closest_hit = None;

    for cube in cubes {
        if let Some(hit) = cube.intersect(ray, t_min, closest) {
            closest = hit.distance;
            closest_hit = Some(hit);
        }
    }

    closest_hit
}

pub(crate) fn hit_color(hit: Intersection) -> Color {
    let normal_color = Color::new(
        hit.normal.x * 0.5 + 0.5,
        hit.normal.y * 0.5 + 0.5,
        hit.normal.z * 0.5 + 0.5,
    );
    let base = Color::new(0.28, 0.42, 0.78);

    (base * 0.35 + normal_color * 0.65).clamped()
}

pub(crate) fn background_color(direction: Vec3) -> Color {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let lower = Color::new(0.06, 0.07, 0.10);
    let upper = Color::new(0.42, 0.55, 0.72);

    lower.lerp(upper, t).clamped()
}

#[cfg(test)]
mod tests {
    use super::{background_color, closest_hit, hit_color, render_background, trace_primary_ray};
    use crate::{
        color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection, math::Vec3,
        ray::Ray,
    };

    fn main_cube() -> Cube {
        Cube::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0), 0)
    }

    #[test]
    fn central_ray_hits_cube() {
        let camera_ray = Ray::new(
            Vec3::new(3.0, 2.0, 5.0),
            Vec3::ZERO - Vec3::new(3.0, 2.0, 5.0),
        );

        assert!(closest_hit(&camera_ray, &[main_cube()], 0.001, 100.0).is_some());
    }

    #[test]
    fn ray_missing_cube_uses_background() {
        let ray = Ray::new(Vec3::new(3.0, 2.0, 5.0), Vec3::new(0.0, 1.0, 0.0));

        assert_eq!(
            trace_primary_ray(&ray, &[main_cube()]).to_u32(),
            background_color(ray.direction).to_u32()
        );
    }

    #[test]
    fn closest_hit_selects_nearest_cube() {
        let near = Cube::new(Vec3::new(-0.5, -0.5, 1.0), Vec3::new(0.5, 0.5, 2.0), 1);
        let far = Cube::new(Vec3::new(-0.5, -0.5, -2.0), Vec3::new(0.5, 0.5, -1.0), 2);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = closest_hit(&ray, &[far, near], 0.001, 100.0).unwrap();

        assert_eq!(hit.material_id, 1);
    }

    #[test]
    fn cube_order_does_not_change_closest_hit() {
        let near = Cube::new(Vec3::new(-0.5, -0.5, 1.0), Vec3::new(0.5, 0.5, 2.0), 1);
        let far = Cube::new(Vec3::new(-0.5, -0.5, -2.0), Vec3::new(0.5, 0.5, -1.0), 2);
        let ray = Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0));
        let first = closest_hit(&ray, &[far, near], 0.001, 100.0).unwrap();
        let second = closest_hit(&ray, &[near, far], 0.001, 100.0).unwrap();

        assert_eq!(first.material_id, second.material_id);
        assert!((first.distance - second.distance).abs() < 0.0001);
    }

    #[test]
    fn hit_color_remains_in_display_range() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = main_cube().intersect(&ray, 0.001, 100.0).unwrap();
        let color = hit_color(hit);

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
        .map(hit_color)
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
