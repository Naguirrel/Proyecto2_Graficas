use crate::{math::Vec3, ray::Ray};

const ORBIT_ROTATION_SPEED: f32 = 1.6;
const ORBIT_ZOOM_SPEED: f32 = 5.0;
const MOUSE_SCROLL_ZOOM_STEP: f32 = 3.0;
const MAX_ORBIT_DELTA_SECONDS: f32 = 0.1;
const MIN_ORBIT_PITCH: f32 = -std::f32::consts::FRAC_PI_2 + 0.05;
const MAX_ORBIT_PITCH: f32 = std::f32::consts::FRAC_PI_2 - 0.05;
const MIN_ORBIT_DISTANCE: f32 = 1.0;
const MAX_ORBIT_DISTANCE: f32 = 30.0;
const DEFAULT_ORBIT_DISTANCE: f32 = 7.3;
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CameraInput {
    pub rotate_left: bool,
    pub rotate_right: bool,
    pub rotate_up: bool,
    pub rotate_down: bool,
    pub zoom_in: bool,
    pub zoom_out: bool,
    pub reset: bool,
    pub scroll_zoom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCameraState {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrbitCamera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub vertical_fov_degrees: f32,
    pub aspect_ratio: f32,
    pub world_up: Vec3,
    pub initial: OrbitCameraState,
}

impl OrbitCameraState {
    pub fn new(target: Vec3, yaw: f32, pitch: f32, distance: f32) -> Self {
        Self {
            target: finite_vec3_or(target, Vec3::ZERO),
            yaw: normalize_yaw(yaw),
            pitch: clamp_pitch(pitch),
            distance: clamp_distance(distance),
        }
    }
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

impl OrbitCamera {
    pub fn new(
        target: Vec3,
        yaw: f32,
        pitch: f32,
        distance: f32,
        vertical_fov_degrees: f32,
        aspect_ratio: f32,
        world_up: Vec3,
    ) -> Self {
        let initial = OrbitCameraState::new(target, yaw, pitch, distance);

        Self {
            target: initial.target,
            yaw: initial.yaw,
            pitch: initial.pitch,
            distance: initial.distance,
            vertical_fov_degrees,
            aspect_ratio,
            world_up,
            initial,
        }
    }

    pub fn position(self) -> Vec3 {
        let pitch_cos = self.pitch.cos();
        let distance = clamp_distance(self.distance);

        self.target
            + Vec3::new(
                distance * pitch_cos * self.yaw.sin(),
                distance * self.pitch.sin(),
                distance * pitch_cos * self.yaw.cos(),
            )
    }

    pub fn to_camera(self) -> Camera {
        Camera::new(
            self.position(),
            self.target,
            self.world_up,
            self.vertical_fov_degrees,
            self.aspect_ratio,
        )
    }

    pub fn update(&mut self, input: CameraInput, delta_seconds: f32) -> bool {
        if delta_seconds <= 0.0 || !delta_seconds.is_finite() {
            return false;
        }

        if input.reset {
            return self.reset();
        }

        let yaw_direction = input.rotate_right as i32 - input.rotate_left as i32;
        let pitch_direction = input.rotate_up as i32 - input.rotate_down as i32;
        let zoom_direction = input.zoom_out as i32 - input.zoom_in as i32;
        let scroll_zoom = if input.scroll_zoom.is_finite() {
            input.scroll_zoom
        } else {
            0.0
        };

        if yaw_direction == 0 && pitch_direction == 0 && zoom_direction == 0 && scroll_zoom == 0.0 {
            return false;
        }

        let previous_yaw = self.yaw;
        let previous_pitch = self.pitch;
        let previous_distance = self.distance;
        let delta_seconds = delta_seconds.min(MAX_ORBIT_DELTA_SECONDS);
        let rotation_step = ORBIT_ROTATION_SPEED * delta_seconds;
        let zoom_step = ORBIT_ZOOM_SPEED * delta_seconds;
        let distance_delta =
            zoom_direction as f32 * zoom_step - scroll_zoom * MOUSE_SCROLL_ZOOM_STEP;

        self.yaw = normalize_yaw(self.yaw + yaw_direction as f32 * rotation_step);
        self.pitch = clamp_pitch(self.pitch + pitch_direction as f32 * rotation_step);
        self.distance = clamp_distance(self.distance + distance_delta);

        (self.yaw - previous_yaw).abs() > 0.0
            || (self.pitch - previous_pitch).abs() > 0.0
            || (self.distance - previous_distance).abs() > 0.0
    }

    pub fn reset(&mut self) -> bool {
        let changed = self.target != self.initial.target
            || self.yaw != self.initial.yaw
            || self.pitch != self.initial.pitch
            || self.distance != self.initial.distance;

        self.target = self.initial.target;
        self.yaw = self.initial.yaw;
        self.pitch = self.initial.pitch;
        self.distance = self.initial.distance;

        changed
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn finite_vec3_or(value: Vec3, fallback: Vec3) -> Vec3 {
    if value.x.is_finite() && value.y.is_finite() && value.z.is_finite() {
        value
    } else {
        fallback
    }
}

fn clamp_distance(distance: f32) -> f32 {
    finite_or(distance, DEFAULT_ORBIT_DISTANCE).clamp(MIN_ORBIT_DISTANCE, MAX_ORBIT_DISTANCE)
}

fn clamp_pitch(pitch: f32) -> f32 {
    finite_or(pitch, 0.0).clamp(MIN_ORBIT_PITCH, MAX_ORBIT_PITCH)
}

fn normalize_yaw(yaw: f32) -> f32 {
    if yaw.is_finite() {
        (yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
    } else {
        0.0
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
    use super::{
        Camera, CameraInput, MAX_ORBIT_DISTANCE, MAX_ORBIT_PITCH, MIN_ORBIT_DISTANCE,
        MIN_ORBIT_PITCH, OrbitCamera,
    };
    use crate::math::Vec3;

    const EPSILON: f32 = 0.0001;

    fn assert_near(left: f32, right: f32) {
        assert!((left - right).abs() < EPSILON, "{left} != {right}");
    }

    fn assert_vec_near(left: Vec3, right: Vec3) {
        assert!(left.approx_eq(right), "{left:?} != {right:?}");
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

    fn default_orbit() -> OrbitCamera {
        OrbitCamera::new(
            Vec3::ZERO,
            0.4,
            0.3,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
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

    #[test]
    fn orbit_position_keeps_distance_to_target() {
        let orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.5,
            0.4,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert_near((orbit.position() - orbit.target).length(), 6.0);
    }

    #[test]
    fn changing_yaw_moves_camera_horizontally() {
        let first = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let second = OrbitCamera::new(
            Vec3::ZERO,
            0.5,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert_ne!(first.position().x, second.position().x);
        assert_ne!(first.position().z, second.position().z);
    }

    #[test]
    fn changing_pitch_changes_height() {
        let first = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let second = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.5,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(second.position().y > first.position().y);
    }

    #[test]
    fn distance_stays_constant_while_rotating() {
        let mut orbit = default_orbit();
        let distance = orbit.distance;

        orbit.update(
            CameraInput {
                rotate_right: true,
                rotate_up: true,
                ..CameraInput::default()
            },
            0.5,
        );

        assert_near((orbit.position() - orbit.target).length(), distance);
    }

    #[test]
    fn full_yaw_turn_returns_to_initial_position() {
        let first = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.2,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let second = OrbitCamera::new(
            Vec3::ZERO,
            std::f32::consts::TAU,
            0.2,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert_vec_near(first.position(), second.position());
    }

    #[test]
    fn orbit_central_ray_points_to_target() {
        let orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.4,
            0.3,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let camera = orbit.to_camera();
        let ray = camera.ray_for_pixel(1, 1, 3, 3);

        assert!(
            ray.direction
                .approx_eq((orbit.target - camera.position).normalized())
        );
    }

    #[test]
    fn orbit_camera_basis_stays_orthonormal() {
        let basis = OrbitCamera::new(
            Vec3::ZERO,
            0.7,
            0.3,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        )
        .to_camera()
        .basis();

        assert_near(basis.forward.length(), 1.0);
        assert_near(basis.right.length(), 1.0);
        assert_near(basis.up.length(), 1.0);
        assert_near(basis.forward.dot(basis.right), 0.0);
        assert_near(basis.forward.dot(basis.up), 0.0);
        assert_near(basis.right.dot(basis.up), 0.0);
    }

    #[test]
    fn pitch_limit_prevents_degeneracy() {
        let orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            10.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert_near(orbit.pitch, MAX_ORBIT_PITCH);
        assert!(orbit.to_camera().basis().right.length() > 0.9);
    }

    #[test]
    fn zooming_in_reduces_distance() {
        let mut orbit = default_orbit();
        let previous_distance = orbit.distance;

        assert!(orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        ));

        assert!(orbit.distance < previous_distance);
    }

    #[test]
    fn zooming_out_increases_distance() {
        let mut orbit = default_orbit();
        let previous_distance = orbit.distance;

        assert!(orbit.update(
            CameraInput {
                zoom_out: true,
                ..CameraInput::default()
            },
            0.25,
        ));

        assert!(orbit.distance > previous_distance);
    }

    #[test]
    fn zoom_respects_minimum_distance() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            MIN_ORBIT_DISTANCE + 0.1,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            1.0,
        );

        assert_near(orbit.distance, MIN_ORBIT_DISTANCE);
    }

    #[test]
    fn zoom_respects_maximum_distance() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            MAX_ORBIT_DISTANCE - 0.1,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        orbit.update(
            CameraInput {
                zoom_out: true,
                ..CameraInput::default()
            },
            1.0,
        );

        assert_near(orbit.distance, MAX_ORBIT_DISTANCE);
    }

    #[test]
    fn invalid_delta_does_not_change_camera() {
        for delta_seconds in [f32::NAN, f32::INFINITY, -0.25] {
            let mut orbit = default_orbit();
            let before = orbit;

            assert!(!orbit.update(
                CameraInput {
                    rotate_right: true,
                    zoom_in: true,
                    reset: true,
                    ..CameraInput::default()
                },
                delta_seconds,
            ));
            assert_eq!(orbit, before);
        }
    }

    #[test]
    fn position_remains_distance_units_from_target_after_zoom() {
        let mut orbit = default_orbit();

        orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        );

        assert_near((orbit.position() - orbit.target).length(), orbit.distance);
    }

    #[test]
    fn zoom_preserves_target_yaw_and_pitch() {
        let mut orbit = default_orbit();
        let previous_target = orbit.target;
        let previous_yaw = orbit.yaw;
        let previous_pitch = orbit.pitch;

        orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        );

        assert_vec_near(orbit.target, previous_target);
        assert_near(orbit.yaw, previous_yaw);
        assert_near(orbit.pitch, previous_pitch);
    }

    #[test]
    fn reset_preserves_aspect_ratio() {
        let mut orbit = default_orbit();
        orbit.aspect_ratio = 16.0 / 9.0;
        orbit.yaw = 1.2;

        assert!(orbit.reset());

        assert_near(orbit.aspect_ratio, 16.0 / 9.0);
    }

    #[test]
    fn central_ray_points_to_target_after_zoom() {
        let mut orbit = default_orbit();

        orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        );

        let camera = orbit.to_camera();
        let ray = camera.ray_for_pixel(1, 1, 3, 3);

        assert!(
            ray.direction
                .approx_eq((orbit.target - camera.position).normalized())
        );
    }

    #[test]
    fn pitch_update_respects_limits() {
        let mut high = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            MAX_ORBIT_PITCH - 0.01,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let mut low = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            MIN_ORBIT_PITCH + 0.01,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        high.update(
            CameraInput {
                rotate_up: true,
                ..CameraInput::default()
            },
            1.0,
        );
        low.update(
            CameraInput {
                rotate_down: true,
                ..CameraInput::default()
            },
            1.0,
        );

        assert_near(high.pitch, MAX_ORBIT_PITCH);
        assert_near(low.pitch, MIN_ORBIT_PITCH);
    }

    #[test]
    fn yaw_remains_stable_after_many_turns() {
        let mut orbit = default_orbit();

        orbit.yaw = std::f32::consts::TAU * 64.0;
        assert!(orbit.update(
            CameraInput {
                rotate_right: true,
                ..CameraInput::default()
            },
            0.25,
        ));

        assert!(orbit.yaw.is_finite());
        assert!(orbit.yaw >= -std::f32::consts::PI);
        assert!(orbit.yaw <= std::f32::consts::PI);
        assert!(orbit.position().x.is_finite());
        assert!(orbit.position().z.is_finite());
    }

    #[test]
    fn reset_restores_yaw() {
        let mut orbit = default_orbit();
        orbit.yaw = 1.2;

        assert!(orbit.reset());

        assert_near(orbit.yaw, orbit.initial.yaw);
    }

    #[test]
    fn reset_restores_pitch() {
        let mut orbit = default_orbit();
        orbit.pitch = -0.7;

        assert!(orbit.reset());

        assert_near(orbit.pitch, orbit.initial.pitch);
    }

    #[test]
    fn reset_restores_distance() {
        let mut orbit = default_orbit();
        orbit.distance = 3.0;

        assert!(orbit.reset());

        assert_near(orbit.distance, orbit.initial.distance);
    }

    #[test]
    fn reset_restores_target() {
        let mut orbit = default_orbit();
        orbit.target = Vec3::new(1.0, 2.0, 3.0);

        assert!(orbit.reset());

        assert_vec_near(orbit.target, orbit.initial.target);
    }

    #[test]
    fn unchanged_reset_does_not_request_render() {
        let mut orbit = default_orbit();

        assert!(!orbit.update(
            CameraInput {
                reset: true,
                ..CameraInput::default()
            },
            0.25,
        ));
    }

    #[test]
    fn effective_zoom_requests_render() {
        let mut orbit = default_orbit();

        assert!(orbit.update(
            CameraInput {
                zoom_in: true,
                ..CameraInput::default()
            },
            0.25,
        ));
    }

    #[test]
    fn effective_reset_requests_render() {
        let mut orbit = default_orbit();
        orbit.yaw = 1.2;

        assert!(orbit.update(
            CameraInput {
                reset: true,
                ..CameraInput::default()
            },
            0.25,
        ));
    }

    #[test]
    fn opposite_zoom_inputs_cancel_each_other() {
        let mut orbit = default_orbit();
        let before = orbit;

        assert!(!orbit.update(
            CameraInput {
                zoom_in: true,
                zoom_out: true,
                ..CameraInput::default()
            },
            0.25,
        ));
        assert_eq!(orbit, before);
    }

    #[test]
    fn zero_delta_does_not_change_camera() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let before = orbit;

        assert!(!orbit.update(
            CameraInput {
                rotate_right: true,
                ..CameraInput::default()
            },
            0.0
        ));
        assert_eq!(orbit, before);
    }

    #[test]
    fn no_input_update_returns_false() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(!orbit.update(CameraInput::default(), 0.25));
    }

    #[test]
    fn valid_input_update_returns_true() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );

        assert!(orbit.update(
            CameraInput {
                rotate_right: true,
                ..CameraInput::default()
            },
            0.25
        ));
    }

    #[test]
    fn opposite_inputs_cancel_each_other() {
        let mut orbit = OrbitCamera::new(
            Vec3::ZERO,
            0.0,
            0.0,
            6.0,
            55.0,
            1.0,
            Vec3::new(0.0, 1.0, 0.0),
        );
        let before = orbit;

        assert!(!orbit.update(
            CameraInput {
                rotate_left: true,
                rotate_right: true,
                rotate_up: true,
                rotate_down: true,
                ..CameraInput::default()
            },
            0.25,
        ));
        assert_eq!(orbit, before);
    }
}
