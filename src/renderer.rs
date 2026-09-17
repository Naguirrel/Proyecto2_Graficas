use crate::{
    camera::Camera,
    color::Color,
    cube::Cube,
    framebuffer::Framebuffer,
    intersection::Intersection,
    light::PointLight,
    material::Material,
    math::{Vec2, Vec3},
    ray::Ray,
    scene::Scene,
    skybox::Skybox,
    texture::{FALLBACK_TEXTURE_COLOR, Texture, WrapMode},
};

#[cfg(test)]
use std::cell::Cell;

const HIT_T_MIN: f32 = 0.001;
const HIT_T_MAX: f32 = 1_000.0;
pub(crate) const MAX_RECURSION_DEPTH: u32 = 4;
const RAY_EPSILON: f32 = 0.001;
const SHADOW_EPSILON: f32 = RAY_EPSILON;
const MIN_REFLECTIVITY: f32 = 0.0001;
const MIN_TRANSPARENCY: f32 = 0.0001;
const SEAT_FABRIC_TEXTURE_PATH: &str = "assets/textures/seat_fabric.ppm";
const THEATER_CARPET_TEXTURE_PATH: &str = "assets/textures/theater_carpet.ppm";
const BRUSHED_METAL_TEXTURE_PATH: &str = "assets/textures/brushed_metal.ppm";
const TRANSPARENT_PLASTIC_TEXTURE_PATH: &str = "assets/textures/transparent_plastic.ppm";
const POPCORN_CARDBOARD_TEXTURE_PATH: &str = "assets/textures/popcorn_cardboard.ppm";
const NIGHT_CINEMA_SKYBOX_TEXTURE_PATH: &str = "assets/textures/night_cinema_skybox.ppm";

#[cfg(test)]
thread_local! {
    static SECONDARY_RAY_COUNT: Cell<usize> = const { Cell::new(0) };
}

pub fn render_background(framebuffer: &mut Framebuffer) {
    let aspect_ratio = framebuffer.width() as f32 / framebuffer.height().max(1) as f32;
    let camera = Camera::new(
        Vec3::new(3.8, 2.6, 5.5),
        Vec3::new(0.0, -0.25, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        55.0,
        aspect_ratio,
    );
    let scene = sample_scene();

    render_scene(framebuffer, &camera, &scene);
}

pub fn render_scene(framebuffer: &mut Framebuffer, camera: &Camera, scene: &Scene) {
    framebuffer.clear(Color::BLACK);

    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            let ray = camera.ray_for_pixel(x, y, framebuffer.width(), framebuffer.height());
            let color = trace_ray(scene, &ray, 0);

            framebuffer.set_pixel(x, y, color);
        }
    }
}

pub(crate) fn sample_scene() -> Scene {
    let mut scene = Scene::new();
    scene.set_ambient_light(Color::new(0.07, 0.065, 0.08));
    let seat_texture = scene.add_texture(load_scene_texture(SEAT_FABRIC_TEXTURE_PATH));
    let carpet_texture = scene.add_texture(load_scene_texture(THEATER_CARPET_TEXTURE_PATH));
    let metal_texture = scene.add_texture(load_scene_texture(BRUSHED_METAL_TEXTURE_PATH));
    let plastic_texture = scene.add_texture(load_scene_texture(TRANSPARENT_PLASTIC_TEXTURE_PATH));
    let cardboard_texture = scene.add_texture(load_scene_texture(POPCORN_CARDBOARD_TEXTURE_PATH));
    scene.set_skybox(load_scene_skybox(NIGHT_CINEMA_SKYBOX_TEXTURE_PATH));

    let seat_fabric = scene
        .add_material(
            Material::new(
                Color::new(0.78, 0.34, 0.34),
                0.22,
                18.0,
                0.03,
                0.0,
                1.0,
                Color::BLACK,
            )
            .with_texture(seat_texture, Vec2::new(3.0, 2.0), WrapMode::Repeat),
        )
        .expect("seat texture was registered before the material");
    let theater_carpet = scene
        .add_material(
            Material::new(
                Color::new(0.42, 0.32, 0.56),
                0.08,
                8.0,
                0.01,
                0.0,
                1.0,
                Color::BLACK,
            )
            .with_texture(carpet_texture, Vec2::new(5.0, 4.0), WrapMode::Repeat),
        )
        .expect("carpet texture was registered before the material");
    let brushed_metal = scene
        .add_material(
            Material::new(
                Color::new(0.76, 0.76, 0.78),
                0.9,
                96.0,
                0.78,
                0.0,
                1.0,
                Color::BLACK,
            )
            .with_texture(metal_texture, Vec2::new(2.0, 1.0), WrapMode::Clamp),
        )
        .expect("metal texture was registered before the material");
    let transparent_plastic = scene
        .add_material(
            Material::new(
                Color::new(0.72, 0.88, 1.0),
                0.55,
                64.0,
                0.35,
                0.72,
                1.49,
                Color::BLACK,
            )
            .with_texture(plastic_texture, Vec2::new(1.5, 1.5), WrapMode::Clamp),
        )
        .expect("plastic texture was registered before the material");
    let popcorn_cardboard = scene
        .add_material(
            Material::new(
                Color::new(1.0, 0.92, 0.72),
                0.18,
                14.0,
                0.02,
                0.0,
                1.0,
                Color::new(0.02, 0.015, 0.005),
            )
            .with_texture(cardboard_texture, Vec2::new(2.0, 2.0), WrapMode::Repeat),
        )
        .expect("cardboard texture was registered before the material");

    scene
        .add_cube(Cube::new(
            Vec3::new(-4.0, -1.15, -4.0),
            Vec3::new(4.0, -1.05, 3.0),
            theater_carpet,
        ))
        .expect("sample floor uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(-3.0, -1.0, -0.9),
            Vec3::new(-2.0, 0.25, 0.35),
            seat_fabric,
        ))
        .expect("seat sample cube uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(-1.45, -1.0, -1.15),
            Vec3::new(-0.45, -0.15, -0.15),
            theater_carpet,
        ))
        .expect("carpet sample cube uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(0.0, -1.0, -0.8),
            Vec3::new(0.85, 0.35, 0.05),
            brushed_metal,
        ))
        .expect("metal sample cube uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(1.95, -1.0, 0.2),
            Vec3::new(2.75, 0.18, 0.95),
            transparent_plastic,
        ))
        .expect("plastic sample cube uses a registered material");
    scene
        .add_cube(Cube::new(
            Vec3::new(2.35, -1.0, -0.65),
            Vec3::new(3.1, 0.55, 0.15),
            popcorn_cardboard,
        ))
        .expect("cardboard sample cube uses a registered material");
    scene.add_light(PointLight::new(
        Vec3::new(-2.7, 4.6, 2.8),
        Color::new(1.0, 0.82, 0.58),
        28.0,
    ));
    scene.add_light(PointLight::new(
        Vec3::new(3.5, 2.4, 4.0),
        Color::new(0.45, 0.62, 1.0),
        5.0,
    ));

    scene
}

fn load_scene_texture(path: &str) -> Texture {
    Texture::from_ppm_file(path).unwrap_or_else(|error| {
        eprintln!("No se pudo cargar {path}: {error:?}. Usando fallback magenta.");
        fallback_texture()
    })
}

fn fallback_texture() -> Texture {
    Texture::new(1, 1, vec![FALLBACK_TEXTURE_COLOR]).expect("fallback texture is valid")
}

fn load_scene_skybox(path: &str) -> Skybox {
    Skybox::from_ppm_file(path)
        .unwrap_or_else(|_| Skybox::new(fallback_texture()))
        .with_intensity(1.15)
        .with_horizontal_rotation(0.08)
}

#[cfg(test)]
pub(crate) fn trace_primary_ray(ray: &Ray, scene: &Scene) -> Color {
    trace_ray(scene, ray, 0)
}

pub(crate) fn trace_ray(scene: &Scene, ray: &Ray, depth: u32) -> Color {
    let Some(hit) = find_nearest_hit(scene, ray) else {
        return background_color_for_scene(scene, ray.direction);
    };
    let material = resolve_material(scene, hit.material_id);
    let surface_albedo = resolve_surface_albedo(scene, material, hit.uv);
    let local_color = shade_hit_with_albedo(ray, hit, material, surface_albedo, scene);

    // At the recursion limit, secondary paths stop and the stable fallback is
    // the already-computed local Phong color.
    if depth >= MAX_RECURSION_DEPTH {
        return local_color;
    }

    compose_secondary_paths(scene, ray, hit, material, local_color, depth)
}

fn find_nearest_hit(scene: &Scene, ray: &Ray) -> Option<Intersection> {
    scene.intersect(ray, HIT_T_MIN, HIT_T_MAX)
}

fn resolve_material(scene: &Scene, material_id: usize) -> Material {
    scene.material(material_id).copied().unwrap_or_default()
}

fn resolve_surface_albedo(scene: &Scene, material: Material, uv: Vec2) -> Color {
    material.albedo * material_texel(scene, material, uv)
}

#[derive(Debug, Clone, Copy)]
struct CompositionWeights {
    local: f32,
    reflection: f32,
    refraction: f32,
}

impl CompositionWeights {
    fn new(material: Material) -> Self {
        Self {
            local: (1.0 - material.transparency) * (1.0 - material.reflectivity),
            reflection: material.reflectivity,
            refraction: material.transparency * (1.0 - material.reflectivity),
        }
    }

    #[cfg(test)]
    fn sum(self) -> f32 {
        self.local + self.reflection + self.refraction
    }
}

fn compose_secondary_paths(
    scene: &Scene,
    ray: &Ray,
    hit: Intersection,
    material: Material,
    local_color: Color,
    depth: u32,
) -> Color {
    let weights = CompositionWeights::new(material);
    let reflected_color = if weights.reflection > MIN_REFLECTIVITY {
        trace_reflection(scene, ray, hit, material, depth)
    } else {
        None
    };
    let refracted_color = if weights.refraction > MIN_TRANSPARENCY {
        trace_refraction(scene, ray, hit, material, depth)
    } else {
        None
    };

    let mut color = local_color * weights.local;

    if let Some(reflected_color) = reflected_color {
        color += reflected_color * weights.reflection;
    } else {
        color += local_color * weights.reflection;
    }

    if let Some(refracted_color) = refracted_color {
        color += refracted_color * weights.refraction;
    } else {
        color += local_color * weights.refraction;
    }

    color.clamped()
}

fn trace_reflection(
    scene: &Scene,
    ray: &Ray,
    hit: Intersection,
    material: Material,
    depth: u32,
) -> Option<Color> {
    if material.reflectivity <= MIN_REFLECTIVITY {
        return None;
    }

    if depth >= MAX_RECURSION_DEPTH {
        return None;
    }

    let reflected_ray = reflected_ray(ray, hit)?;

    record_secondary_ray();
    Some(trace_ray(scene, &reflected_ray, depth + 1))
}

fn trace_refraction(
    scene: &Scene,
    ray: &Ray,
    hit: Intersection,
    material: Material,
    depth: u32,
) -> Option<Color> {
    if material.transparency <= MIN_TRANSPARENCY {
        return None;
    }

    if depth >= MAX_RECURSION_DEPTH {
        return None;
    }

    match refracted_ray(ray, hit, material) {
        Some(ray) => {
            record_secondary_ray();
            Some(trace_ray(scene, &ray, depth + 1))
        }
        None => {
            let reflected_ray = reflected_ray(ray, hit)?;

            record_secondary_ray();
            Some(trace_ray(scene, &reflected_ray, depth + 1))
        }
    }
}

fn reflected_ray(ray: &Ray, hit: Intersection) -> Option<Ray> {
    let incident_direction = ray.direction.normalized();
    let reflected_direction = incident_direction.reflect(hit.normal).normalized();

    if reflected_direction == Vec3::ZERO {
        return None;
    }

    let offset_direction = if reflected_direction.dot(hit.normal) >= 0.0 {
        hit.normal
    } else {
        -hit.normal
    };
    let reflected_origin = hit.position + offset_direction * RAY_EPSILON;

    Some(Ray::new(reflected_origin, reflected_direction))
}

fn refracted_ray(ray: &Ray, hit: Intersection, material: Material) -> Option<Ray> {
    let incident_direction = ray.direction.normalized();
    let outward_normal = hit.normal.normalized();

    if incident_direction == Vec3::ZERO || outward_normal == Vec3::ZERO {
        return None;
    }

    let entering = incident_direction.dot(outward_normal) < 0.0;
    let (n1, n2, formula_normal) = if entering {
        (1.0, material.refractive_index, outward_normal)
    } else {
        (material.refractive_index, 1.0, -outward_normal)
    };
    let eta_ratio = n1 / n2;
    let refracted_direction = refract(incident_direction, formula_normal, eta_ratio)?;
    let offset_direction = if refracted_direction.dot(outward_normal) >= 0.0 {
        outward_normal
    } else {
        -outward_normal
    };
    let refracted_origin = hit.position + offset_direction * RAY_EPSILON;

    Some(Ray::new(refracted_origin, refracted_direction))
}

/// Refracts `incident` through a surface with `normal` opposing the incoming ray.
/// `eta_ratio` is n1 / n2, where n1 is the current medium's refractive index and
/// n2 is the destination medium's refractive index. Returns `None` for total
/// internal reflection or invalid inputs.
pub(crate) fn refract(incident: Vec3, normal: Vec3, eta_ratio: f32) -> Option<Vec3> {
    if !eta_ratio.is_finite() || eta_ratio <= 0.0 {
        return None;
    }

    let incident = incident.normalized();
    let normal = normal.normalized();

    if incident == Vec3::ZERO || normal == Vec3::ZERO {
        return None;
    }

    let cos_theta = (-incident).dot(normal).clamp(0.0, 1.0);
    let perpendicular = (incident + normal * cos_theta) * eta_ratio;
    let parallel_squared = 1.0 - perpendicular.length_squared();

    if parallel_squared < -0.0001 {
        return None;
    }

    let parallel = normal * -parallel_squared.max(0.0).sqrt();
    let refracted = (perpendicular + parallel).normalized();

    if refracted == Vec3::ZERO {
        None
    } else {
        Some(refracted)
    }
}

#[cfg(test)]
fn record_secondary_ray() {
    SECONDARY_RAY_COUNT.with(|count| count.set(count.get() + 1));
}

#[cfg(not(test))]
fn record_secondary_ray() {}

#[cfg(test)]
fn reset_secondary_ray_count() {
    SECONDARY_RAY_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn secondary_ray_count() -> usize {
    SECONDARY_RAY_COUNT.with(Cell::get)
}

pub(crate) fn material_texel(scene: &Scene, material: Material, uv: Vec2) -> Color {
    let Some(texture_id) = material.texture_id else {
        return Color::WHITE;
    };
    let Some(texture) = scene.texture(texture_id) else {
        return FALLBACK_TEXTURE_COLOR;
    };
    let scaled_uv = Vec2::new(uv.u * material.uv_scale.u, uv.v * material.uv_scale.v);

    texture.sample(scaled_uv, material.wrap_mode)
}

/// Local Phong shading: emission + ambient + Lambert diffuse + Phong specular.
/// Hard shadows skip only a blocked light's diffuse and specular terms.
#[cfg(test)]
pub(crate) fn shade_hit(ray: &Ray, hit: Intersection, material: Material, scene: &Scene) -> Color {
    shade_hit_with_albedo(ray, hit, material, material.albedo, scene)
}

pub(crate) fn shade_hit_with_albedo(
    ray: &Ray,
    hit: Intersection,
    material: Material,
    surface_albedo: Color,
    scene: &Scene,
) -> Color {
    let mut color = material.emission + surface_albedo * scene.ambient_light();
    let view_direction = (-ray.direction).normalized();

    for light in scene.lights() {
        let light_offset = light.position - hit.position;
        let distance_squared = light_offset.length_squared();
        let light_direction = light_offset.normalized();
        let diffuse_factor = hit.normal.dot(light_direction).max(0.0);

        if light.intensity <= 0.0 || light_direction == Vec3::ZERO {
            continue;
        }

        if !is_light_visible(scene, &hit, light) {
            continue;
        }

        let attenuation = light.intensity / (1.0 + distance_squared.max(0.0001));

        if diffuse_factor > 0.0 {
            let diffuse = surface_albedo * light.color * (diffuse_factor * attenuation);
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

pub(crate) fn is_light_visible(scene: &Scene, hit: &Intersection, light: &PointLight) -> bool {
    let light_offset = light.position - hit.position;
    let distance_to_light = light_offset.length();

    if !distance_to_light.is_finite() || distance_to_light <= SHADOW_EPSILON {
        return true;
    }

    let light_direction = light_offset / distance_to_light;

    if light_direction == Vec3::ZERO {
        return true;
    }

    // Offset away from the surface. Use the geometric normal when the light is
    // on the normal side; otherwise step along the light ray to avoid nudging
    // the origin into the surface.
    let offset_direction = if hit.normal.dot(light_direction) >= 0.0 {
        hit.normal
    } else {
        light_direction
    };
    let shadow_origin = hit.position + offset_direction * SHADOW_EPSILON;
    let shadow_ray = Ray::new(shadow_origin, light_direction);
    let shadow_t_max = distance_to_light - SHADOW_EPSILON;

    if shadow_t_max <= SHADOW_EPSILON || !shadow_t_max.is_finite() {
        return true;
    }

    scene
        .intersect(&shadow_ray, SHADOW_EPSILON, shadow_t_max)
        .is_none()
}

pub(crate) fn background_color(direction: Vec3) -> Color {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let lower = Color::new(0.06, 0.07, 0.10);
    let upper = Color::new(0.42, 0.55, 0.72);

    lower.lerp(upper, t).clamped()
}

pub(crate) fn background_color_for_scene(scene: &Scene, direction: Vec3) -> Color {
    scene
        .skybox()
        .map(|skybox| skybox.sample_direction(direction))
        .unwrap_or_else(|| background_color(direction))
}

#[cfg(test)]
mod tests {
    use super::{
        CompositionWeights, MAX_RECURSION_DEPTH, RAY_EPSILON, background_color,
        background_color_for_scene, is_light_visible, material_texel, reflected_ray, refract,
        refracted_ray, render_background, render_scene, reset_secondary_ray_count, sample_scene,
        secondary_ray_count, shade_hit, shade_hit_with_albedo, trace_primary_ray, trace_ray,
        trace_refraction,
    };
    use crate::{
        camera::{Camera, OrbitCamera},
        color::Color,
        cube::Cube,
        framebuffer::Framebuffer,
        intersection::Intersection,
        light::PointLight,
        material::Material,
        math::{Vec2, Vec3},
        ray::Ray,
        scene::Scene,
        skybox::Skybox,
        texture::{FALLBACK_TEXTURE_COLOR, Texture, WrapMode},
    };

    fn scene_with_main_cube() -> Scene {
        let mut scene = Scene::new();
        let material_id = scene
            .add_material(Material::diffuse(Color::new(0.28, 0.42, 0.78)))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();
        scene
    }

    fn scene_with_material(material: Material) -> Scene {
        let mut scene = Scene::new();
        scene.add_material(material).unwrap();
        scene
    }

    fn corner_texture() -> Texture {
        Texture::new(
            2,
            2,
            vec![
                Color::new(1.0, 0.0, 0.0),
                Color::new(0.0, 1.0, 0.0),
                Color::new(0.0, 0.0, 1.0),
                Color::new(1.0, 1.0, 1.0),
            ],
        )
        .unwrap()
    }

    fn solid_skybox(color: Color) -> Skybox {
        Skybox::new(Texture::new(1, 1, vec![color]).unwrap())
    }

    fn cardinal_skybox() -> Skybox {
        Skybox::new(
            Texture::new(
                4,
                3,
                vec![
                    Color::new(0.0, 0.0, 0.3),
                    Color::new(0.0, 0.0, 0.4),
                    Color::new(0.0, 0.0, 0.5),
                    Color::new(0.0, 0.0, 0.3),
                    Color::new(1.0, 0.0, 0.0),
                    Color::new(0.0, 1.0, 0.0),
                    Color::new(0.0, 0.0, 1.0),
                    Color::new(1.0, 0.0, 0.0),
                    Color::new(0.2, 0.0, 0.0),
                    Color::new(0.0, 0.2, 0.0),
                    Color::new(0.0, 0.0, 0.2),
                    Color::new(0.2, 0.0, 0.0),
                ],
            )
            .unwrap(),
        )
    }

    fn flat_hit() -> Intersection {
        Intersection::new(
            1.0,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(0.5, 0.5),
            0,
        )
    }

    fn view_ray() -> Ray {
        Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0))
    }

    fn occluder(material_id: usize) -> Cube {
        Cube::new(
            Vec3::new(-0.25, -0.25, 1.5),
            Vec3::new(0.25, 0.25, 2.0),
            material_id,
        )
    }

    fn assert_color_near(left: Color, right: Color) {
        assert!(
            (left.r - right.r).abs() < 0.0001
                && (left.g - right.g).abs() < 0.0001
                && (left.b - right.b).abs() < 0.0001,
            "{left:?} != {right:?}"
        );
    }

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < 0.0001, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
    }

    fn assert_finite_unit_color(color: Color) {
        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
    }

    fn reflective_material(reflectivity: f32) -> Material {
        Material::new(
            Color::new(0.2, 0.4, 0.6),
            0.0,
            1.0,
            reflectivity,
            0.0,
            1.0,
            Color::BLACK,
        )
    }

    fn front_cube_scene(reflectivity: f32) -> Scene {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::WHITE);
        let material_id = scene
            .add_material(reflective_material(reflectivity))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();
        scene
    }

    fn transparent_material(transparency: f32, reflectivity: f32) -> Material {
        Material::new(
            Color::new(0.2, 0.4, 0.6),
            0.0,
            1.0,
            reflectivity,
            transparency,
            1.5,
            Color::BLACK,
        )
    }

    fn transparent_cube_scene(transparency: f32, reflectivity: f32) -> Scene {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::WHITE);
        let material_id = scene
            .add_material(transparent_material(transparency, reflectivity))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();
        scene
    }

    fn front_ray() -> Ray {
        Ray::new(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0))
    }

    fn transparent_target_scene(textured_target: bool) -> Scene {
        let mut scene = transparent_cube_scene(1.0, 0.0);
        let target_material = if textured_target {
            let texture_id =
                scene.add_texture(Texture::new(1, 1, vec![Color::new(0.9, 0.1, 0.2)]).unwrap());

            Material::diffuse(Color::WHITE).with_texture(
                texture_id,
                Vec2::new(1.0, 1.0),
                WrapMode::Clamp,
            )
        } else {
            Material::diffuse(Color::new(0.9, 0.1, 0.2))
        };
        let target_id = scene.add_material(target_material).unwrap();

        scene
            .add_cube(Cube::new(
                Vec3::new(-0.65, -0.65, -2.5),
                Vec3::new(0.65, 0.65, -1.5),
                target_id,
            ))
            .unwrap();

        scene
    }

    fn reflection_target_ray() -> Ray {
        Ray::new(Vec3::new(0.0, 3.0, 3.0), Vec3::new(0.0, -2.0, -3.0))
    }

    fn reflection_target_scene(reflectivity: f32, textured_target: bool) -> Scene {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::WHITE);
        let mirror_id = scene
            .add_material(Material::new(
                Color::BLACK,
                0.0,
                1.0,
                reflectivity,
                0.0,
                1.0,
                Color::BLACK,
            ))
            .unwrap();
        let target_material = if textured_target {
            let texture_id =
                scene.add_texture(Texture::new(1, 1, vec![Color::new(0.9, 0.1, 0.2)]).unwrap());

            Material::diffuse(Color::WHITE).with_texture(
                texture_id,
                Vec2::new(1.0, 1.0),
                WrapMode::Clamp,
            )
        } else {
            Material::diffuse(Color::new(0.9, 0.1, 0.2))
        };
        let target_id = scene.add_material(target_material).unwrap();

        scene
            .add_cube(Cube::new(
                Vec3::new(-0.55, 0.0, -0.55),
                Vec3::new(0.55, 1.0, 0.55),
                mirror_id,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.55, 1.25, -1.5),
                Vec3::new(0.55, 2.3, -0.7),
                target_id,
            ))
            .unwrap();

        scene
    }

    fn facing_mirrors_scene() -> Scene {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        let material_id = scene
            .add_material(Material::new(
                Color::new(0.1, 0.1, 0.1),
                0.0,
                1.0,
                1.0,
                0.0,
                1.0,
                Color::BLACK,
            ))
            .unwrap();

        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -3.0),
                Vec3::new(1.0, 1.0, -2.0),
                material_id,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, 2.0),
                Vec3::new(1.0, 1.0, 3.0),
                material_id,
            ))
            .unwrap();

        scene
    }

    fn red_channel(pixel: u32) -> u8 {
        ((pixel >> 16) & 0xff) as u8
    }

    #[test]
    fn light_is_visible_without_obstacle() {
        let scene = scene_with_material(Material::diffuse(Color::WHITE));
        let hit = flat_hit();
        let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn cube_between_hit_and_light_blocks_light() {
        let material = Material::diffuse(Color::WHITE);
        let mut scene = scene_with_material(material);
        scene.add_cube(occluder(0)).unwrap();
        let hit = flat_hit();
        let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::WHITE, 1.0);

        assert!(!is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn cube_behind_light_does_not_cast_shadow() {
        let material = Material::diffuse(Color::WHITE);
        let mut scene = scene_with_material(material);
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.25, -0.25, 3.5),
                Vec3::new(0.25, 0.25, 4.0),
                0,
            ))
            .unwrap();
        let hit = flat_hit();
        let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn cube_behind_hit_does_not_cast_shadow() {
        let material = Material::diffuse(Color::WHITE);
        let mut scene = scene_with_material(material);
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.25, -0.25, -2.0),
                Vec3::new(0.25, 0.25, -1.5),
                0,
            ))
            .unwrap();
        let hit = flat_hit();
        let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn own_surface_does_not_self_occlude() {
        let material = Material::diffuse(Color::WHITE);
        let mut scene = scene_with_material(material);
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                0,
            ))
            .unwrap();
        let hit = Intersection::new(
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(0.5, 0.5),
            0,
        );
        let light = PointLight::new(Vec3::new(0.0, 0.0, 3.0), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn offset_prevents_acne_on_lit_surface() {
        let material = Material::diffuse(Color::WHITE);
        let mut scene = scene_with_material(material);
        scene
            .add_cube(Cube::new(
                Vec3::new(-10.0, -0.1, -10.0),
                Vec3::new(10.0, 0.0, 10.0),
                0,
            ))
            .unwrap();
        let hit = Intersection::new(
            1.0,
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec2::new(0.5, 0.5),
            0,
        );
        let light = PointLight::new(Vec3::new(0.0, 2.0, 0.0), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
    }

    #[test]
    fn extremely_close_light_is_handled_safely() {
        let scene = scene_with_main_cube();
        let hit = flat_hit();
        let light = PointLight::new(Vec3::new(0.0, 0.0, 0.00001), Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
        let color = shade_hit(&view_ray(), hit, Material::diffuse(Color::WHITE), &scene);
        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
    }

    #[test]
    fn invalid_shadow_interval_does_not_panic() {
        let scene = scene_with_main_cube();
        let hit = flat_hit();
        let light = PointLight::new(hit.position, Color::WHITE, 1.0);

        assert!(is_light_visible(&scene, &hit, &light));
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
    fn untextured_material_uses_neutral_white_texel() {
        let scene = Scene::new();
        let material = Material::diffuse(Color::new(0.2, 0.3, 0.4));

        assert_eq!(
            material_texel(&scene, material, Vec2::new(0.5, 0.5)),
            Color::WHITE
        );
    }

    #[test]
    fn invalid_material_texture_uses_fallback_texel() {
        let scene = Scene::new();
        let material =
            Material::diffuse(Color::WHITE).with_texture(10, Vec2::new(1.0, 1.0), WrapMode::Clamp);

        assert_eq!(
            material_texel(&scene, material, Vec2::new(0.5, 0.5)),
            FALLBACK_TEXTURE_COLOR
        );
    }

    #[test]
    fn repeat_sampling_respects_material_uv_scale() {
        let mut scene = Scene::new();
        let texture_id = scene.add_texture(corner_texture());
        let material = Material::diffuse(Color::WHITE).with_texture(
            texture_id,
            Vec2::new(2.0, 2.0),
            WrapMode::Repeat,
        );

        assert_eq!(
            material_texel(&scene, material, Vec2::new(0.875, 0.875)),
            Color::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn clamp_sampling_respects_material_wrap_mode() {
        let mut scene = Scene::new();
        let texture_id = scene.add_texture(corner_texture());
        let material = Material::diffuse(Color::WHITE).with_texture(
            texture_id,
            Vec2::new(2.0, 2.0),
            WrapMode::Clamp,
        );

        assert_eq!(
            material_texel(&scene, material, Vec2::new(0.875, 0.875)),
            Color::new(0.0, 1.0, 0.0)
        );
    }

    #[test]
    fn final_albedo_combines_albedo_and_texel() {
        let mut scene = Scene::new();
        let texture_id =
            scene.add_texture(Texture::new(1, 1, vec![Color::new(0.5, 0.25, 1.0)]).unwrap());
        let material = Material::diffuse(Color::new(0.4, 0.8, 0.2)).with_texture(
            texture_id,
            Vec2::new(1.0, 1.0),
            WrapMode::Repeat,
        );
        let surface_albedo = material.albedo * material_texel(&scene, material, Vec2::ZERO);

        assert_eq!(surface_albedo, Color::new(0.2, 0.2, 0.2));
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
    fn scene_without_skybox_keeps_gradient_background() {
        let scene = Scene::new();
        let direction = Vec3::new(0.0, 1.0, 0.0);

        assert_color_near(
            background_color_for_scene(&scene, direction),
            background_color(direction),
        );
    }

    #[test]
    fn missed_primary_ray_uses_skybox_when_present() {
        let mut scene = Scene::new();
        let skybox_color = Color::new(0.7, 0.2, 0.1);
        scene.set_skybox(solid_skybox(skybox_color));
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0));

        assert_color_near(trace_ray(&scene, &ray, 0), skybox_color);
    }

    #[test]
    fn missed_reflected_ray_uses_skybox() {
        let mut scene = front_cube_scene(1.0);
        let skybox_color = Color::new(0.1, 0.7, 0.3);
        scene.set_skybox(solid_skybox(skybox_color));

        assert_color_near(trace_ray(&scene, &front_ray(), 0), skybox_color);
    }

    #[test]
    fn refracted_ray_exiting_to_environment_uses_skybox() {
        let mut scene = transparent_cube_scene(1.0, 0.0);
        let skybox_color = Color::new(0.2, 0.3, 0.8);
        scene.set_skybox(solid_skybox(skybox_color));

        assert_color_near(trace_ray(&scene, &front_ray(), 0), skybox_color);
    }

    #[test]
    fn reflective_metal_can_show_skybox_color() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        scene.set_skybox(solid_skybox(Color::new(0.3, 0.6, 0.9)));
        let metal_id = scene
            .add_material(Material::new(
                Color::BLACK,
                0.9,
                96.0,
                1.0,
                0.0,
                1.0,
                Color::BLACK,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                metal_id,
            ))
            .unwrap();

        assert_color_near(
            trace_ray(&scene, &front_ray(), 0),
            Color::new(0.3, 0.6, 0.9),
        );
    }

    #[test]
    fn transparent_material_can_show_skybox_color() {
        let mut scene = transparent_cube_scene(1.0, 0.0);
        let skybox_color = Color::new(0.6, 0.4, 0.2);
        scene.set_skybox(solid_skybox(skybox_color));

        assert_color_near(trace_ray(&scene, &front_ray(), 0), skybox_color);
    }

    #[test]
    fn vec3_reflect_matches_trace_ray_convention() {
        let incident = Vec3::new(1.0, -1.0, 0.0).normalized();
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let formula = incident - normal * (2.0 * incident.dot(normal));

        assert!(incident.reflect(normal).approx_eq(formula));
        assert!(
            incident
                .reflect(normal)
                .approx_eq(Vec3::new(1.0, 1.0, 0.0).normalized())
        );
    }

    #[test]
    fn perpendicular_refraction_preserves_direction() {
        let incident = Vec3::new(0.0, 0.0, -1.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let refracted = refract(incident, normal, 1.0 / 1.5).unwrap();

        assert_vec_near(refracted, incident);
    }

    #[test]
    fn air_to_glass_refraction_produces_valid_direction() {
        let incident = Vec3::new(0.5, 0.0, -1.0).normalized();
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let refracted = refract(incident, normal, 1.0 / 1.5).unwrap();

        assert!(refracted.x.is_finite());
        assert!(refracted.y.is_finite());
        assert!(refracted.z.is_finite());
        assert_near(refracted.length(), 1.0);
    }

    #[test]
    fn glass_to_air_refraction_produces_valid_direction_when_angle_allows() {
        let incident = Vec3::new(0.25, 0.0, -1.0).normalized();
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let refracted = refract(incident, normal, 1.5).unwrap();

        assert!(refracted.x.is_finite());
        assert!(refracted.y.is_finite());
        assert!(refracted.z.is_finite());
        assert_near(refracted.length(), 1.0);
    }

    #[test]
    fn air_to_glass_bends_toward_normal() {
        let incident = Vec3::new(1.0, 0.0, -1.0).normalized();
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let refracted = refract(incident, normal, 1.0 / 1.5).unwrap();

        assert!(refracted.dot(-normal) > incident.dot(-normal));
    }

    #[test]
    fn glass_to_air_bends_away_from_normal() {
        let incident = Vec3::new(0.3, 0.0, -1.0).normalized();
        let normal = Vec3::new(0.0, 0.0, 1.0);

        let refracted = refract(incident, normal, 1.5).unwrap();

        assert!(refracted.dot(-normal) < incident.dot(-normal));
    }

    #[test]
    fn total_internal_reflection_returns_none() {
        let incident = Vec3::new(0.9, 0.0, -0.3).normalized();
        let normal = Vec3::new(0.0, 0.0, 1.0);

        assert_eq!(refract(incident, normal, 1.5), None);
    }

    #[test]
    fn refracted_direction_is_normalized() {
        let incident = Vec3::new(0.4, 0.0, -1.0);
        let normal = Vec3::new(0.0, 0.0, 3.0);

        let refracted = refract(incident, normal, 1.0 / 1.5).unwrap();

        assert_near(refracted.length(), 1.0);
    }

    #[test]
    fn invalid_refraction_inputs_return_none_without_nan() {
        assert_eq!(
            refract(
                Vec3::new(f32::NAN, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
                1.0 / 1.5
            ),
            None
        );
        assert_eq!(refract(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), 1.0), None);
        assert_eq!(refract(Vec3::new(0.0, 0.0, -1.0), Vec3::ZERO, 1.0), None);
        assert_eq!(
            refract(
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
                f32::INFINITY
            ),
            None
        );
    }

    #[test]
    fn reflectivity_zero_preserves_local_color_without_secondary_rays() {
        let scene = front_cube_scene(0.0);
        let ray = front_ray();
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let local_color = shade_hit(&ray, hit, *scene.material(hit.material_id).unwrap(), &scene);

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, local_color);
        assert_eq!(secondary_ray_count(), 0);
    }

    #[test]
    fn reflectivity_one_uses_reflected_background_when_ray_misses() {
        let scene = front_cube_scene(1.0);
        let ray = front_ray();
        let expected_background = background_color(Vec3::new(0.0, 0.0, 1.0));

        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, expected_background);
    }

    #[test]
    fn intermediate_reflectivity_blends_local_and_reflected_color() {
        let scene = front_cube_scene(0.25);
        let ray = front_ray();
        let local_color = Color::new(0.2, 0.4, 0.6);
        let reflected_color = background_color(Vec3::new(0.0, 0.0, 1.0));
        let expected = local_color * 0.75 + reflected_color * 0.25;

        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, expected);
    }

    #[test]
    fn transparency_zero_preserves_previous_opaque_result() {
        let scene = transparent_cube_scene(0.0, 0.0);
        let ray = front_ray();
        let local_color = Color::new(0.2, 0.4, 0.6);

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, local_color);
        assert_eq!(secondary_ray_count(), 0);
    }

    #[test]
    fn transparency_one_uses_refracted_background() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let ray = front_ray();
        let expected_background = background_color(ray.direction);

        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, expected_background);
    }

    #[test]
    fn intermediate_transparency_blends_local_and_refracted_color() {
        let scene = transparent_cube_scene(0.5, 0.0);
        let ray = front_ray();
        let local_color = Color::new(0.2, 0.4, 0.6);
        let refracted_color = background_color(ray.direction);
        let expected = local_color * 0.75 + refracted_color * 0.25;

        let color = trace_ray(&scene, &ray, 0);

        assert_color_near(color, expected);
    }

    #[test]
    fn reflectivity_and_transparency_weights_are_non_negative() {
        let weights = CompositionWeights::new(Material::new(
            Color::WHITE,
            0.0,
            1.0,
            0.35,
            0.72,
            1.5,
            Color::BLACK,
        ));

        assert!(weights.local >= 0.0);
        assert!(weights.reflection >= 0.0);
        assert!(weights.refraction >= 0.0);
    }

    #[test]
    fn composition_weights_sum_to_one() {
        let weights = CompositionWeights::new(Material::new(
            Color::WHITE,
            0.0,
            1.0,
            0.35,
            0.72,
            1.5,
            Color::BLACK,
        ));

        assert_near(weights.sum(), 1.0);
    }

    #[test]
    fn refracted_ray_enters_and_exits_transparent_cube() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let ray = front_ray();
        let entry_hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let inside_ray = refracted_ray(
            &ray,
            entry_hit,
            *scene.material(entry_hit.material_id).unwrap(),
        )
        .unwrap();
        let exit_hit = scene.intersect(&inside_ray, RAY_EPSILON, 100.0).unwrap();

        assert_eq!(entry_hit.normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(exit_hit.normal, Vec3::new(0.0, 0.0, -1.0));
        assert!(inside_ray.origin.z < entry_hit.position.z);
    }

    #[test]
    fn exiting_face_uses_glass_to_air_eta_ratio() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let material = *scene.material(0).unwrap();
        let ray = Ray::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.25, 0.0, -1.0));
        let exit_hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let outgoing = refracted_ray(&ray, exit_hit, material).unwrap();
        let expected = refract(ray.direction, -exit_hit.normal, material.refractive_index).unwrap();

        assert_eq!(exit_hit.normal, Vec3::new(0.0, 0.0, -1.0));
        assert_vec_near(outgoing.direction, expected);
    }

    #[test]
    fn refracted_origin_is_offset_into_destination_medium() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let ray = front_ray();
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let refracted =
            refracted_ray(&ray, hit, *scene.material(hit.material_id).unwrap()).unwrap();

        assert!(refracted.origin.z < hit.position.z);
        assert_color_near(
            Color::new(hit.position.z - refracted.origin.z, 0.0, 0.0),
            Color::new(RAY_EPSILON, 0.0, 0.0),
        );
        assert_ne!(
            scene
                .intersect(&refracted, RAY_EPSILON, 100.0)
                .unwrap()
                .normal,
            hit.normal
        );
    }

    #[test]
    fn total_internal_reflection_uses_reflected_color_for_refraction_path() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let material = *scene.material(0).unwrap();
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.7, 0.0, -0.714));
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let reflected = reflected_ray(&ray, hit).unwrap();
        let reflected_color = trace_ray(&scene, &reflected, 1);
        let refracted_color = trace_refraction(&scene, &ray, hit, material, 0).unwrap();

        assert_eq!(refracted_ray(&ray, hit, material), None);
        assert_color_near(refracted_color, reflected_color);
    }

    #[test]
    fn background_is_visible_through_transparent_material() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let ray = front_ray();

        assert_color_near(trace_ray(&scene, &ray, 0), background_color(ray.direction));
    }

    #[test]
    fn object_is_visible_through_transparent_cube() {
        let scene = transparent_target_scene(false);

        let color = trace_ray(&scene, &front_ray(), 0);

        assert!(color.r > 0.8, "{color:?}");
        assert!(color.g < 0.2, "{color:?}");
        assert!(color.b < 0.3, "{color:?}");
    }

    #[test]
    fn textured_object_remains_visible_through_transparent_cube() {
        let scene = transparent_target_scene(true);

        let color = trace_ray(&scene, &front_ray(), 0);

        assert_color_near(color, Color::new(0.9, 0.1, 0.2));
    }

    #[test]
    fn opaque_material_matches_local_shading_after_transparency_refactor() {
        let scene = scene_with_main_cube();
        let ray = front_ray();
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let local_color = shade_hit(&ray, hit, *scene.material(hit.material_id).unwrap(), &scene);

        assert_color_near(trace_ray(&scene, &ray, 0), local_color);
    }

    #[test]
    fn max_recursion_depth_stops_refraction_paths() {
        let scene = transparent_cube_scene(1.0, 0.0);
        let ray = front_ray();

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &ray, MAX_RECURSION_DEPTH);

        assert_color_near(color, Color::new(0.2, 0.4, 0.6));
        assert_eq!(secondary_ray_count(), 0);
    }

    #[test]
    fn overlapping_transparent_objects_do_not_recurse_forever() {
        let mut scene = transparent_cube_scene(1.0, 0.0);
        scene
            .add_cube(Cube::new(
                Vec3::new(-0.75, -0.75, -1.75),
                Vec3::new(0.75, 0.75, 0.25),
                0,
            ))
            .unwrap();

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &front_ray(), 0);

        assert_finite_unit_color(color);
        assert!(secondary_ray_count() <= MAX_RECURSION_DEPTH as usize);
    }

    #[test]
    fn transparent_result_contains_no_nan_or_infinity_and_stays_clamped() {
        let scene = transparent_target_scene(true);
        let color = trace_ray(&scene, &front_ray(), 0);

        assert_finite_unit_color(color);
    }

    #[test]
    fn reflected_ray_can_hit_another_object() {
        let scene = reflection_target_scene(1.0, false);
        let color = trace_ray(&scene, &reflection_target_ray(), 0);

        assert!(color.r > 0.8, "{color:?}");
        assert!(color.g < 0.2, "{color:?}");
        assert!(color.b < 0.3, "{color:?}");
    }

    #[test]
    fn reflected_object_keeps_its_texture() {
        let scene = reflection_target_scene(1.0, true);
        let color = trace_ray(&scene, &reflection_target_ray(), 0);

        assert_color_near(color, Color::new(0.9, 0.1, 0.2));
    }

    #[test]
    fn reflected_origin_is_offset_away_from_hit_surface() {
        let scene = front_cube_scene(1.0);
        let ray = front_ray();
        let hit = scene.intersect(&ray, 0.001, 100.0).unwrap();
        let reflected = reflected_ray(&ray, hit).unwrap();

        assert!(reflected.origin.z > hit.position.z);
        assert_color_near(
            Color::new(reflected.origin.z - hit.position.z, 0.0, 0.0),
            Color::new(RAY_EPSILON, 0.0, 0.0),
        );
        assert!(scene.intersect(&reflected, RAY_EPSILON, 100.0).is_none());
    }

    #[test]
    fn different_view_angles_generate_different_reflection_directions() {
        let hit = flat_hit();
        let straight = reflected_ray(&view_ray(), hit).unwrap();
        let angled = reflected_ray(
            &Ray::new(Vec3::new(1.0, 0.0, 4.0), Vec3::new(-0.25, 0.0, -1.0)),
            hit,
        )
        .unwrap();

        assert_ne!(straight.direction, angled.direction);
    }

    #[test]
    fn max_recursion_depth_returns_stable_local_color() {
        let scene = front_cube_scene(1.0);
        let ray = front_ray();

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &ray, MAX_RECURSION_DEPTH);

        assert_color_near(color, Color::new(0.2, 0.4, 0.6));
        assert_eq!(secondary_ray_count(), 0);
    }

    #[test]
    fn facing_reflective_surfaces_stop_at_recursion_limit() {
        let scene = facing_mirrors_scene();
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));

        reset_secondary_ray_count();
        let color = trace_ray(&scene, &ray, 0);

        assert_finite_unit_color(color);
        assert!(secondary_ray_count() <= MAX_RECURSION_DEPTH as usize);
    }

    #[test]
    fn reflected_result_contains_no_nan_or_infinity_and_stays_clamped() {
        let scene = reflection_target_scene(0.6, true);
        let color = trace_ray(&scene, &reflection_target_ray(), 0);

        assert_finite_unit_color(color);
    }

    #[test]
    fn trace_ray_preserves_emission_in_local_shading() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        let material_id = scene
            .add_material(Material::new(
                Color::BLACK,
                0.0,
                1.0,
                0.0,
                0.0,
                1.0,
                Color::new(0.2, 0.3, 0.4),
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                material_id,
            ))
            .unwrap();

        assert_color_near(
            trace_ray(&scene, &front_ray(), 0),
            Color::new(0.2, 0.3, 0.4),
        );
    }

    #[test]
    fn trace_ray_keeps_shadowed_local_shading() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        let surface_id = scene
            .add_material(Material::new(
                Color::WHITE,
                0.0,
                1.0,
                0.0,
                0.0,
                1.0,
                Color::BLACK,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(-1.0, -1.0, -1.0),
                Vec3::new(1.0, 1.0, 1.0),
                surface_id,
            ))
            .unwrap();
        scene
            .add_cube(Cube::new(
                Vec3::new(0.2, -0.2, 1.7),
                Vec3::new(0.45, 0.2, 2.1),
                surface_id,
            ))
            .unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.6, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_color_near(trace_ray(&scene, &front_ray(), 0), Color::BLACK);
    }

    #[test]
    fn two_materials_can_produce_different_colors() {
        let scene = sample_scene();
        let first = Intersection::new(
            1.0,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(0.5, 0.5),
            0,
        );
        let second = Intersection::new(
            1.0,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(0.5, 0.5),
            1,
        );
        let ray = view_ray();

        assert_ne!(
            shade_hit(&ray, first, *scene.material(0).unwrap(), &scene).to_u32(),
            shade_hit(&ray, second, *scene.material(1).unwrap(), &scene).to_u32()
        );
    }

    #[test]
    fn sample_scene_uses_five_distinct_textured_materials() {
        let scene = sample_scene();
        let materials = scene.materials();

        assert_eq!(scene.textures().len(), 5);
        assert!(scene.skybox().is_some());
        assert_eq!(materials.len(), 5);
        let mut texture_ids = materials
            .iter()
            .map(|material| material.texture_id.unwrap())
            .collect::<Vec<_>>();
        texture_ids.sort_unstable();
        texture_ids.dedup();
        assert_eq!(texture_ids.len(), 5);

        for material in materials {
            assert!((0.0..=1.0).contains(&material.specular_strength));
            assert!((0.0..=1.0).contains(&material.reflectivity));
            assert!((0.0..=1.0).contains(&material.transparency));
            assert!(material.shininess >= 0.0);
            assert!(material.refractive_index >= 1.0);
        }
    }

    #[test]
    fn sample_scene_material_parameters_match_roles() {
        let scene = sample_scene();
        let materials = scene.materials();
        let seat = materials[0];
        let carpet = materials[1];
        let metal = materials[2];
        let plastic = materials[3];
        let cardboard = materials[4];

        assert_eq!(seat.transparency, 0.0);
        assert!(seat.reflectivity < 0.1);
        assert_eq!(carpet.transparency, 0.0);
        assert!(carpet.specular_strength < 0.1);
        assert!(carpet.reflectivity < 0.05);
        assert!(metal.specular_strength > 0.8);
        assert!(metal.reflectivity > 0.7);
        assert!(plastic.transparency > 0.7);
        assert!(plastic.reflectivity > 0.3);
        assert!(plastic.refractive_index >= 1.49);
        assert_eq!(cardboard.transparency, 0.0);
        assert!(cardboard.reflectivity < 0.1);
    }

    #[test]
    fn scene_intersection_selects_nearest_cube_for_renderer() {
        let mut scene = Scene::new();
        let near_id = scene
            .add_material(Material::diffuse(Color::new(0.7, 0.2, 0.2)))
            .unwrap();
        let far_id = scene
            .add_material(Material::diffuse(Color::new(0.2, 0.2, 0.7)))
            .unwrap();
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
    fn textured_albedo_still_receives_lighting() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::new(0.1, 0.1, 0.1));
        scene.add_light(PointLight::new(Vec3::new(0.0, 0.0, 2.0), Color::WHITE, 3.0));
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);
        let surface_albedo = Color::new(0.25, 0.5, 0.75);

        let color =
            shade_hit_with_albedo(&view_ray(), flat_hit(), material, surface_albedo, &scene);

        assert!(color.r > surface_albedo.r * scene.ambient_light().r);
        assert!(color.g > surface_albedo.g * scene.ambient_light().g);
        assert!(color.b > surface_albedo.b * scene.ambient_light().b);
    }

    #[test]
    fn textured_shading_preserves_shadows() {
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::BLACK);
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_eq!(
            shade_hit_with_albedo(
                &view_ray(),
                flat_hit(),
                material,
                Color::new(0.25, 0.5, 0.75),
                &scene,
            ),
            Color::BLACK
        );
    }

    #[test]
    fn emission_survives_textured_albedo() {
        let mut scene = Scene::new();
        scene.set_ambient_light(Color::BLACK);
        let material = Material::new(
            Color::WHITE,
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.1, 0.2, 0.3),
        );

        assert_eq!(
            shade_hit_with_albedo(&view_ray(), flat_hit(), material, Color::BLACK, &scene),
            material.emission
        );
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
    fn emission_remains_when_light_is_blocked() {
        let material = Material::new(
            Color::BLACK,
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::new(0.2, 0.1, 0.05),
        );
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::BLACK);
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            material.emission
        );
    }

    #[test]
    fn ambient_remains_when_light_is_blocked() {
        let material = Material::new(
            Color::new(0.5, 0.25, 0.75),
            0.0,
            1.0,
            0.0,
            0.0,
            1.0,
            Color::BLACK,
        );
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::new(0.2, 0.2, 0.2));
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            Color::new(0.1, 0.05, 0.15)
        );
    }

    #[test]
    fn blocked_light_removes_diffuse_component() {
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::BLACK);
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            Color::BLACK
        );
    }

    #[test]
    fn blocked_light_removes_specular_component() {
        let material = Material::new(Color::BLACK, 1.0, 32.0, 0.0, 0.0, 1.0, Color::BLACK);
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::BLACK);
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));

        assert_eq!(
            shade_hit(&view_ray(), flat_hit(), material, &scene),
            Color::BLACK
        );
    }

    #[test]
    fn visible_light_still_contributes_when_another_light_is_blocked() {
        let material = Material::new(Color::WHITE, 0.0, 1.0, 0.0, 0.0, 1.0, Color::BLACK);
        let mut scene = scene_with_material(Material::diffuse(Color::WHITE));
        scene.set_ambient_light(Color::BLACK);
        scene.add_cube(occluder(0)).unwrap();
        scene.add_light(PointLight::new(
            Vec3::new(0.0, 0.0, 3.0),
            Color::WHITE,
            10.0,
        ));
        scene.add_light(PointLight::new(Vec3::new(2.0, 0.0, 2.0), Color::WHITE, 3.0));

        assert!(shade_hit(&view_ray(), flat_hit(), material, &scene).r > 0.0);
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
    fn render_small_framebuffer_contains_texture_variation() {
        let mut scene = Scene::new();
        let texture_id = scene.add_texture(corner_texture());
        let material_id = scene
            .add_material(Material::diffuse(Color::WHITE).with_texture(
                texture_id,
                Vec2::new(1.0, 1.0),
                WrapMode::Clamp,
            ))
            .unwrap();
        scene.set_ambient_light(Color::WHITE);
        scene
            .add_cube(Cube::new(
                Vec3::new(-2.0, -2.0, -1.0),
                Vec3::new(2.0, 2.0, 1.0),
                material_id,
            ))
            .unwrap();
        let mut framebuffer = Framebuffer::new(64, 64);
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 4.0),
            Vec3::ZERO,
            Vec3::new(0.0, 1.0, 0.0),
            55.0,
            1.0,
        );

        render_scene(&mut framebuffer, &camera, &scene);

        let first = framebuffer.pixels()[8 * 64 + 8];
        let second = framebuffer.pixels()[8 * 64 + 56];
        let third = framebuffer.pixels()[56 * 64 + 8];
        let fourth = framebuffer.pixels()[56 * 64 + 56];

        assert_ne!(first, second);
        assert_ne!(first, third);
        assert_ne!(second, fourth);
    }

    #[test]
    fn small_framebuffer_contains_detectable_reflection() {
        let reflective_scene = reflection_target_scene(1.0, true);
        let matte_scene = reflection_target_scene(0.0, true);
        let camera = Camera::new(
            Vec3::new(0.0, 3.0, 3.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            45.0,
            1.0,
        );
        let mut reflective = Framebuffer::new(1, 1);
        let mut matte = Framebuffer::new(1, 1);

        render_scene(&mut reflective, &camera, &reflective_scene);
        render_scene(&mut matte, &camera, &matte_scene);

        assert_eq!(reflective.width(), 1);
        assert_eq!(reflective.height(), 1);
        assert_eq!(reflective.pixels().len(), 1);
        assert!(red_channel(reflective.pixels()[0]) > red_channel(matte.pixels()[0]));
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

    #[test]
    fn framebuffer_with_skybox_keeps_dimensions() {
        let mut scene = Scene::new();
        scene.set_skybox(cardinal_skybox());
        let mut framebuffer = Framebuffer::new(7, 5);
        let camera = Camera::new(
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            framebuffer.width() as f32 / framebuffer.height() as f32,
        );

        render_scene(&mut framebuffer, &camera, &scene);

        assert_eq!(framebuffer.width(), 7);
        assert_eq!(framebuffer.height(), 5);
        assert_eq!(framebuffer.pixels().len(), 35);
    }

    #[test]
    fn skybox_rendered_framebuffer_has_valid_pixels() {
        let mut scene = Scene::new();
        scene.set_skybox(cardinal_skybox());
        let mut framebuffer = Framebuffer::new(4, 3);
        let camera = Camera::new(
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            framebuffer.width() as f32 / framebuffer.height() as f32,
        );

        render_scene(&mut framebuffer, &camera, &scene);

        assert!(
            framebuffer
                .pixels()
                .iter()
                .all(|&pixel| pixel <= 0x00ff_ffff)
        );
    }

    #[test]
    fn orbit_camera_changes_visible_skybox_region() {
        let mut scene = Scene::new();
        scene.set_skybox(cardinal_skybox());
        let mut first = Framebuffer::new(1, 1);
        let mut second = Framebuffer::new(1, 1);
        let first_camera = Camera::new(
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
        );
        let second_camera = Camera::new(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            1.0,
        );

        render_scene(&mut first, &first_camera, &scene);
        render_scene(&mut second, &second_camera, &scene);

        assert_ne!(first.pixels(), second.pixels());
    }

    #[test]
    fn sample_scene_framebuffer_contains_multiple_material_colors() {
        let scene = sample_scene();
        let mut framebuffer = Framebuffer::new(80, 48);
        let aspect_ratio = framebuffer.width() as f32 / framebuffer.height() as f32;
        let camera = Camera::new(
            Vec3::new(3.8, 2.6, 5.5),
            Vec3::new(0.0, -0.25, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            55.0,
            aspect_ratio,
        );

        render_scene(&mut framebuffer, &camera, &scene);

        let mut distinct = Vec::new();
        for &pixel in framebuffer.pixels() {
            if !distinct.contains(&pixel) {
                distinct.push(pixel);
            }
            if distinct.len() >= 8 {
                break;
            }
        }

        assert!(distinct.len() >= 8);
    }

    #[test]
    fn framebuffer_changes_after_orbit_rotation() {
        let scene = sample_scene();
        let mut first = Framebuffer::new(48, 36);
        let mut second = Framebuffer::new(48, 36);
        let aspect_ratio = first.width() as f32 / first.height() as f32;
        let target = Vec3::new(0.0, -0.25, 0.0);
        let first_camera = OrbitCamera::new(
            target,
            0.6,
            0.4,
            7.3,
            55.0,
            aspect_ratio,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera();
        let second_camera = OrbitCamera::new(
            target,
            1.1,
            0.4,
            7.3,
            55.0,
            aspect_ratio,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera();

        render_scene(&mut first, &first_camera, &scene);
        render_scene(&mut second, &second_camera, &scene);

        assert_ne!(first.pixels(), second.pixels());
    }

    #[test]
    fn framebuffer_changes_after_orbit_zoom() {
        let scene = sample_scene();
        let mut first = Framebuffer::new(48, 36);
        let mut second = Framebuffer::new(48, 36);
        let aspect_ratio = first.width() as f32 / first.height() as f32;
        let target = Vec3::new(0.0, -0.25, 0.0);
        let first_camera = OrbitCamera::new(
            target,
            0.6,
            0.4,
            7.3,
            55.0,
            aspect_ratio,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera();
        let second_camera = OrbitCamera::new(
            target,
            0.6,
            0.4,
            5.3,
            55.0,
            aspect_ratio,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera();

        render_scene(&mut first, &first_camera, &scene);
        render_scene(&mut second, &second_camera, &scene);

        assert_eq!(first.width(), second.width());
        assert_eq!(first.height(), second.height());
        assert_eq!(first.pixels().len(), second.pixels().len());
        assert_ne!(first.pixels(), second.pixels());
    }

    #[test]
    fn framebuffer_dimensions_survive_orbit_render() {
        let scene = sample_scene();
        let mut framebuffer = Framebuffer::new(48, 36);
        let aspect_ratio = framebuffer.width() as f32 / framebuffer.height() as f32;
        let camera = OrbitCamera::new(
            Vec3::new(0.0, -0.25, 0.0),
            0.9,
            0.35,
            7.3,
            55.0,
            aspect_ratio,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera();

        render_scene(&mut framebuffer, &camera, &scene);

        assert_eq!(framebuffer.width(), 48);
        assert_eq!(framebuffer.height(), 36);
        assert_eq!(framebuffer.pixels().len(), 48 * 36);
    }
}
