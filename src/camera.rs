use crate::{math::Vec3, ray::Ray};

const DEFAULT_FORWARD: Vec3 = Vec3::new(0.0, 0.0, -1.0);
const DEFAULT_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const SIDE_UP: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const DEFAULT_FOV_DEGREES: f32 = 60.0;
const MIN_FOV_DEGREES: f32 = 1.0;
const MAX_FOV_DEGREES: f32 = 179.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub world_up: Vec3,
    /// Vertical field of view in degrees.
    pub vertical_fov_degrees: f32,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraBasis {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(
        position: Vec3,
        target: Vec3,
        world_up: Vec3,
        vertical_fov_degrees: f32,
        aspect_ratio: f32,
    ) -> Self {
        Self {
            position,
            target,
            world_up,
            vertical_fov_degrees,
            aspect_ratio,
        }
    }

    pub fn basis(self) -> CameraBasis {
        let forward = safe_forward(self.target - self.position);
        let up_hint = safe_up(self.world_up, forward);
        let right = forward.cross(up_hint).normalized();
        let right = if right == Vec3::ZERO {
            forward.cross(fallback_up(forward)).normalized()
        } else {
            right
        };
        let up = right.cross(forward).normalized();

        CameraBasis { forward, right, up }
    }

    pub fn ray_for_pixel(
        self,
        pixel_x: usize,
        pixel_y: usize,
        framebuffer_width: usize,
        framebuffer_height: usize,
    ) -> Ray {
        let basis = self.basis();
        let (u, v) =
            normalized_pixel_coordinates(pixel_x, pixel_y, framebuffer_width, framebuffer_height);
        let half_height = (self.valid_vertical_fov_degrees().to_radians() * 0.5).tan();
        let half_width = half_height * self.valid_aspect_ratio();
        let direction =
            basis.forward + basis.right * (u * half_width) + basis.up * (v * half_height);

        Ray::new(self.position, direction)
    }

    fn valid_vertical_fov_degrees(self) -> f32 {
        if self.vertical_fov_degrees.is_finite() {
            self.vertical_fov_degrees
                .clamp(MIN_FOV_DEGREES, MAX_FOV_DEGREES)
        } else {
            DEFAULT_FOV_DEGREES
        }
    }

    fn valid_aspect_ratio(self) -> f32 {
        if self.aspect_ratio.is_finite() && self.aspect_ratio > 0.0 {
            self.aspect_ratio
        } else {
            1.0
        }
    }
}

fn safe_forward(direction: Vec3) -> Vec3 {
    let forward = direction.normalized();

    if forward == Vec3::ZERO {
        DEFAULT_FORWARD
    } else {
        forward
    }
}

fn safe_up(world_up: Vec3, forward: Vec3) -> Vec3 {
    let up = world_up.normalized();
    let up = if up == Vec3::ZERO { DEFAULT_UP } else { up };

    if forward.cross(up).length_squared() <= 0.0001 {
        fallback_up(forward)
    } else {
        up
    }
}

fn fallback_up(forward: Vec3) -> Vec3 {
    if forward.dot(DEFAULT_UP).abs() > 0.9 {
        SIDE_UP
    } else {
        DEFAULT_UP
    }
}

fn normalized_pixel_coordinates(
    pixel_x: usize,
    pixel_y: usize,
    framebuffer_width: usize,
    framebuffer_height: usize,
) -> (f32, f32) {
    let width = framebuffer_width.max(1) as f32;
    let height = framebuffer_height.max(1) as f32;
    let u = ((pixel_x as f32 + 0.5) / width) * 2.0 - 1.0;
    let v = 1.0 - ((pixel_y as f32 + 0.5) / height) * 2.0;

    (u, v)
}

#[cfg(test)]
mod tests {
    use super::Camera;
    use crate::math::Vec3;

    const EPSILON: f32 = 0.0001;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
    }

    fn default_camera(aspect_ratio: f32) -> Camera {
        Camera::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.0, 1.0, 0.0),
            60.0,
            aspect_ratio,
        )
    }

    #[test]
    fn central_ray_points_at_target() {
        let ray = default_camera(1.0).ray_for_pixel(1, 1, 3, 3);

        assert!(ray.direction.approx_eq(Vec3::new(0.0, 0.0, -1.0)));
    }

    #[test]
    fn corner_rays_have_distinct_directions() {
        let camera = default_camera(1.0);
        let top_left = camera.ray_for_pixel(0, 0, 4, 4);
        let bottom_right = camera.ray_for_pixel(3, 3, 4, 4);

        assert!(!top_left.direction.approx_eq(bottom_right.direction));
    }

    #[test]
    fn basis_is_approximately_orthonormal() {
        let basis = default_camera(1.0).basis();

        assert_near(basis.forward.length(), 1.0);
        assert_near(basis.right.length(), 1.0);
        assert_near(basis.up.length(), 1.0);
        assert_near(basis.forward.dot(basis.right), 0.0);
        assert_near(basis.forward.dot(basis.up), 0.0);
        assert_near(basis.right.dot(basis.up), 0.0);
    }

    #[test]
    fn aspect_ratio_expands_horizontal_extent() {
        let square = default_camera(1.0).ray_for_pixel(0, 1, 3, 3);
        let wide = default_camera(2.0).ray_for_pixel(0, 1, 3, 3);

        assert!(wide.direction.x.abs() > square.direction.x.abs());
        assert_near(wide.direction.y, square.direction.y);
    }

    #[test]
    fn degenerate_inputs_do_not_create_nan() {
        let camera = Camera::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::ZERO,
            f32::NAN,
            -1.0,
        );
        let ray = camera.ray_for_pixel(0, 0, 1, 1);

        assert!(ray.direction.x.is_finite());
        assert!(ray.direction.y.is_finite());
        assert!(ray.direction.z.is_finite());
        assert!(ray.direction.approx_eq(Vec3::new(0.0, 0.0, -1.0)));
    }

    #[test]
    fn parallel_world_up_still_builds_valid_basis() {
        let camera = Camera::new(
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            60.0,
            1.0,
        );
        let basis = camera.basis();

        assert_near(basis.right.length(), 1.0);
        assert_near(basis.up.length(), 1.0);
        assert_near(basis.forward.dot(basis.up), 0.0);
    }

    #[test]
    fn uses_pixel_center_for_sampling() {
        let ray = default_camera(1.0).ray_for_pixel(0, 0, 2, 2);

        assert!(ray.direction.x < 0.0);
        assert!(ray.direction.y > 0.0);
        assert!(ray.direction.z < 0.0);
    }

    #[test]
    fn vertical_framebuffer_axis_points_downward() {
        let camera = default_camera(1.0);
        let top = camera.ray_for_pixel(1, 0, 3, 3);
        let bottom = camera.ray_for_pixel(1, 2, 3, 3);

        assert!(top.direction.y > 0.0);
        assert!(bottom.direction.y < 0.0);
    }
}
