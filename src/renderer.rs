use crate::{
    camera::Camera, color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection,
    light::PointLight, material::Material, math::Vec3, ray::Ray, scene::Scene,
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
    scene.set_ambient_light(Color::new(0.07, 0.065, 0.08));
    let blue = scene.add_material(Material::new(
        Color::new(0.28, 0.42, 0.78),
        0.75,
        48.0,
        0.0,
        0.0,
        1.0,
        Color::BLACK,
    ));
    let red = scene.add_material(Material::new(
        Color::new(0.72, 0.22, 0.18),
        0.25,
        18.0,
        0.0,
        0.0,
        1.0,
        Color::new(0.02, 0.0, 0.0),
    ));

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
    scene.add_light(PointLight::new(
        Vec3::new(3.5, 4.5, 2.5),
        Color::new(1.0, 0.82, 0.58),
        18.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(-4.0, 2.5, 3.0),
        Color::new(0.45, 0.62, 1.0),
        4.5,
    ));

    scene
}

pub(crate) fn trace_primary_ray(ray: &Ray, scene: &Scene) -> Color {
    match scene.intersect(ray, HIT_T_MIN, HIT_T_MAX) {
        Some(hit) => {
            let material = scene.material(hit.material_id).copied().unwrap_or_default();
            shade_hit(ray, hit, material, scene)
        }
        None => background_color(ray.direction),
    }
}

/// Local Phong shading: emission + ambient + Lambert diffuse + Phong specular.
/// Shadows and secondary rays are intentionally omitted in this sprint step.
pub(crate) fn shade_hit(ray: &Ray, hit: Intersection, material: Material, scene: &Scene) -> Color {
    let mut color = material.emission + material.albedo * scene.ambient_light();
    let view_direction = (-ray.direction).normalized();

    for light in scene.lights() {
        let light_offset = light.position - hit.position;
        let distance_squared = light_offset.length_squared();
        let light_direction = light_offset.normalized();
        let diffuse_factor = hit.normal.dot(light_direction).max(0.0);

        if light.intensity <= 0.0 || light_direction == Vec3::ZERO {
            continue;
        }

        let attenuation = light.intensity / (1.0 + distance_squared.max(0.0001));

        if diffuse_factor > 0.0 {
            let diffuse = material.albedo * light.color * (diffuse_factor * attenuation);
            let reflected_light = (-light_direction).reflect(hit.normal).normalized();
            let specular_factor = reflected_light
                .dot(view_direction)
                .max(0.0)
                .powf(material.shininess)
                * material.specular_strength
                * attenuation;
            let specular = light.color * specular_factor;

            color += diffuse + specular;
        }
    }

    color.clamped()
}

pub(crate) fn background_color(direction: Vec3) -> Color {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let lower = Color::new(0.06, 0.07, 0.10);
    let upper = Color::new(0.42, 0.55, 0.72);

    lower.lerp(upper, t).clamped()
}

#[cfg(test)]
mod tests {
    use super::{background_color, render_background, sample_scene, shade_hit, trace_primary_ray};
    use crate::{
        color::Color, cube::Cube, framebuffer::Framebuffer, intersection::Intersection,
        light::PointLight, material::Material, math::Vec3, ray::Ray, scene::Scene,
    };

    fn scene_with_main_cube() -> Scene {
        let mut scene = Scene::new();
        let material_id = scene.add_material(Material::diffuse(Color::new(0.28, 0.42, 0.78)));
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();
        scene
    }

    fn flat_hit() -> Intersection {
        Intersection::new(1.0, Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), 0)
    }

    fn view_ray() -> Ray {
        Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0))
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
        let ray = view_ray();

        assert_ne!(
            shade_hit(&ray, first, *scene.material(0).unwrap(), &scene).to_u32(),
            shade_hit(&ray, second, *scene.material(1).unwrap(), &scene).to_u32()
        );
    }

    #[test]
    fn scene_intersection_selects_nearest_cube_for_renderer() {
        let mut scene = Scene::new();
        let near_id = scene.add_material(Material::diffuse(Color::new(0.7, 0.2, 0.2)));
        let far_id = scene.add_material(Material::diffuse(Color::new(0.2, 0.2, 0.7)));
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
    fn shaded_color_remains_in_display_range() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0));
        let scene = scene_with_main_cube();
        let hit = scene.cubes()[0].intersect(&ray, 0.001, 100.0).unwrap();
        let color = shade_hit(&ray, hit, *scene.material(hit.material_id).unwrap(), &scene);

        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
    }

    #[test]
    fn no_lights_keeps_ambient_and_emission() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::new(0.1, 0.2, 0.3));
        let material = Material::new(
            Color::new(0.5, 0.5, 0.5),
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.05, 0.04, 0.03),
        );

        let color = shade_hit(&view_ray(), flat_hit(), material, &scene);

        assert_eq!(color, Color::new(0.1, 0.14, 0.18));
    }

    #[test]
    fn surface_facing_light_receives_diffuse_light() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 1.0), Color::WHITE, 2.0));
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);

        let color = shade_hit(&view_ray(), flat_hit(), material, &scene);

        assert!(color.r > 0.0);
        assert!(color.g > 0.0);
        assert!(color.b > 0.0);
    }

    #[test]
    fn surface_opposite_light_gets_no_diffuse_light() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, -1.0),
            Color::WHITE,
            3.0,
        ));
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            Color::BLACK
        );
    }

    #[test]
    fn zero_specular_strength_removes_specular_highlight() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 1.0), Color::WHITE, 2.0));
        let diffuse_only = Material::new(Color::BLACK, 0.0, 32.0, 0.0, 0.0, 1.0, Color::BLACK);

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), diffuse_only, &scene),
            Color::BLACK
        );
    }

    #[test]
    fn shiny_material_produces_stronger_specular_highlight() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 1.0), Color::WHITE, 2.0));
        let matte = Material::new(Color::BLACK, 0.2, 32.0, 0.0, 0.0, 1.0, Color::BLACK);
        let shiny = Material::new(Color::BLACK, 0.9, 32.0, 0.0, 0.0, 1.0, Color::BLACK);

        assert!(
            shade_hit(&view_ray(), flat_hit(), shiny, &scene).r
                > shade_hit(&view_ray(), flat_hit(), matte, &scene).r
        );
    }

    #[test]
    fn distant_light_contributes_less() {
        let mut near_scene = Scene::new();
        let mut far_scene = Scene::new();
        near_scene.set_ambient_light(Color::BLACK);
        far_scene.set_ambient_light(Color::BLACK);
        near_scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 1.0), Color::WHITE, 2.0));
        far_scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 4.0), Color::WHITE, 2.0));
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);

        assert!(
            shade_hit(&view_ray(), flat_hit(), material, &near_scene).r
                > shade_hit(&view_ray(), flat_hit(), material, &far_scene).r
        );
    }

    #[test]
    fn two_lights_sum_their_contributions() {
        let mut one_light = Scene::new();
        let mut two_lights = Scene::new();
        one_light.set_ambient_light(Color::BLACK);
        two_lights.set_ambient_light(Color::BLACK);
        let light = PointLight::new(Vec3::new(0.0, 0.0, 1.0), Color::WHITE, 1.0);
        one_light.add_light(light);
        two_lights.add_light(light);
        two_lights.add_light(light);
        let material = Material::new(
            Color::new(0.5, 0.5, 0.5),
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::BLACK,
        );

        assert!(
            shade_hit(&view_ray(), flat_hit(), material, &two_lights).r
                > shade_hit(&view_ray(), flat_hit(), material, &one_light).r
        );
    }

    #[test]
    fn coincident_light_does_not_produce_nan() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.add_light(PointLight::new(Vec3::ZERO, Color::WHITE, 10.0));
        let material = Material::new(Color::WHITE, 1.0, 32.0, 0.0, 0.0, 1.0, Color::BLACK);
        let color = shade_hit(&view_ray(), flat_hit(), material, &scene);

        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
    }

    #[test]
    fn emission_is_visible_without_lights() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        let material = Material::new(
            Color::BLACK,
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.2, 0.3, 0.4),
        );

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            material.emission
        );
    }

    #[test]
    fn render_small_framebuffer_contains_cube_and_background_pixels() {
        let mut framebuffer = Framebuffer::new(64, 48);

        render_background(&mut framebuffer);

        let center_pixel = framebuffer.pixels()[24 * 64 + 32];
        let corner_pixel = framebuffer.pixels()[0];

        assert_ne!(center_pixel, corner_pixel);
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
