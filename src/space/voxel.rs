//! Voxel planets: spheres rebuilt from small axis-aligned cubes.
//!
//! A `VoxelBody` is the union of a few balls laid over a regular grid of
//! cubes with a cube corner at the first ball's center, so the poles of a
//! ball end in a flat patch of cubes instead of a lone cube. A cell belongs
//! to the body when its center lies inside any ball, and it takes the
//! material of the last ball that contains it, so later balls (like soil
//! mounds) paint over earlier ones. Only cells with an empty face neighbor go to the scene: any
//! ray coming from outside hits one of them first, so the hidden inside costs
//! nothing and transparent bodies stay hollow.
//!
//! Each ball keeps the material of the sphere it replaces. A textured ball is
//! sampled at the direction of every cell (the same sphere UVs the sphere
//! used) and those colors are reduced to a small palette of plain materials,
//! so every cube has one flat color, like a voxel model. A ball without a
//! texture (glass, for example) keeps its material as it is.

use std::{error::Error, fmt};

use super::SpaceBuildError;
use crate::{
    color::Color, cube::Cube, material::Material, math::Vec3, renderer::material_texel,
    scene::Scene, sphere::Sphere,
};

/// Colors per textured material of a body.
const VOXEL_PALETTE_SIZE: usize = 8;
const PALETTE_ITERATIONS: usize = 12;
/// Each cube fills this fraction of its cell, so thin dark seams show where
/// the cubes meet.
pub(super) const VOXEL_CUBE_FILL: f32 = 0.94;
/// Opaque balls get a dark core sphere this many cube edges under their
/// surface. It fills the seams, so they never show the empty inside.
const CORE_DEPTH_CUBES: f32 = 1.5;
/// `surface_distance` walks this many steps per cube edge.
const SURFACE_STEPS_PER_CUBE: f32 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelError {
    InvalidCubeEdge,
    NoBalls,
}

impl fmt::Display for VoxelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCubeEdge => write!(formatter, "voxel cube edge must be finite and > 0"),
            Self::NoBalls => write!(formatter, "a voxel body needs at least one ball"),
        }
    }
}

impl Error for VoxelError {}

/// One ball of a voxel body, with the material of the sphere it replaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct VoxelBall {
    pub center: Vec3,
    pub radius: f32,
    pub material_id: usize,
}

impl VoxelBall {
    pub(super) const fn new(center: Vec3, radius: f32, material_id: usize) -> Self {
        Self {
            center,
            radius,
            material_id,
        }
    }

    fn contains(self, point: Vec3) -> bool {
        (point - self.center).length_squared() <= self.radius * self.radius
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VoxelBody {
    balls: Vec<VoxelBall>,
    cube_edge: f32,
}

/// Where the cubes of a body went in the scene.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VoxelBodyParts {
    /// Id of the first cube; the body's cubes are consecutive objects.
    pub first_id: usize,
    pub cube_count: usize,
    /// Dark core spheres added right after the cubes.
    pub core_count: usize,
    /// Plain materials registered for the palettes of textured balls.
    pub palette_materials: usize,
}

impl VoxelBodyParts {
    /// Ids of the body's cubes.
    #[cfg(test)]
    pub(crate) fn ids(self) -> std::ops::Range<usize> {
        self.first_id..self.first_id + self.cube_count
    }

    /// Cubes plus cores.
    pub(crate) fn total(self) -> usize {
        self.cube_count + self.core_count
    }
}

/// A cell of the body that has at least one empty face neighbor.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SurfaceCell {
    center: Vec3,
    /// Index of the ball that paints this cell.
    ball: usize,
}

impl VoxelBody {
    pub(super) fn new(balls: &[VoxelBall], cube_edge: f32) -> Result<Self, SpaceBuildError> {
        if !cube_edge.is_finite() || cube_edge <= 0.0 {
            return Err(VoxelError::InvalidCubeEdge.into());
        }
        if balls.is_empty() {
            return Err(VoxelError::NoBalls.into());
        }
        for ball in balls {
            // Same center and radius checks as the sphere it replaces.
            Sphere::new(ball.center, ball.radius, ball.material_id)?;
        }

        Ok(Self {
            balls: balls.to_vec(),
            cube_edge,
        })
    }

    /// A single ball whose cube edge is `cube_edge`, or smaller so that at
    /// least `min_cubes_across` cubes span its diameter.
    pub(super) fn ball(
        ball: VoxelBall,
        cube_edge: f32,
        min_cubes_across: f32,
    ) -> Result<Self, SpaceBuildError> {
        Self::new(
            &[ball],
            fitted_cube_edge(ball.radius, cube_edge, min_cubes_across),
        )
    }

    pub(super) fn origin(&self) -> Vec3 {
        self.balls[0].center
    }

    fn cell_center(&self, cell: [i32; 3]) -> Vec3 {
        self.origin()
            + Vec3::new(
                cell[0] as f32 + 0.5,
                cell[1] as f32 + 0.5,
                cell[2] as f32 + 0.5,
            ) * self.cube_edge
    }

    /// Ball that paints the cell centered at `point`: the last one that
    /// contains it.
    fn owner(&self, point: Vec3) -> Option<usize> {
        self.balls.iter().rposition(|ball| ball.contains(point))
    }

    #[cfg(test)]
    fn contains_cell(&self, cell: [i32; 3]) -> bool {
        self.owner(self.cell_center(cell)).is_some()
    }

    /// Distance from `from` along `direction` to the top of the body's cubes
    /// (see `surface_distance`).
    pub(super) fn surface_distance(&self, from: Vec3, direction: Vec3) -> f32 {
        let shape: Vec<(Vec3, f32)> = self
            .balls
            .iter()
            .map(|ball| (ball.center, ball.radius))
            .collect();

        surface_distance(&shape, self.cube_edge, from, direction)
    }

    /// Cells of the body with at least one empty face neighbor, in a fixed
    /// order (by x, then y, then z).
    fn surface_cells(&self) -> Vec<SurfaceCell> {
        let (low, high) = self.cell_bounds();
        let size = [
            (high[0] - low[0] + 1) as usize,
            (high[1] - low[1] + 1) as usize,
            (high[2] - low[2] + 1) as usize,
        ];
        let index = |x: usize, y: usize, z: usize| (x * size[1] + y) * size[2] + z;
        let mut owners = vec![None; size[0] * size[1] * size[2]];

        for x in 0..size[0] {
            for y in 0..size[1] {
                for z in 0..size[2] {
                    let cell = [low[0] + x as i32, low[1] + y as i32, low[2] + z as i32];
                    owners[index(x, y, z)] = self.owner(self.cell_center(cell));
                }
            }
        }

        let filled = |x: i64, y: i64, z: i64| {
            x >= 0
                && y >= 0
                && z >= 0
                && (x as usize) < size[0]
                && (y as usize) < size[1]
                && (z as usize) < size[2]
                && owners[index(x as usize, y as usize, z as usize)].is_some()
        };
        let mut cells = Vec::new();

        for x in 0..size[0] {
            for y in 0..size[1] {
                for z in 0..size[2] {
                    let Some(ball) = owners[index(x, y, z)] else {
                        continue;
                    };
                    let (xi, yi, zi) = (x as i64, y as i64, z as i64);
                    let hidden = filled(xi - 1, yi, zi)
                        && filled(xi + 1, yi, zi)
                        && filled(xi, yi - 1, zi)
                        && filled(xi, yi + 1, zi)
                        && filled(xi, yi, zi - 1)
                        && filled(xi, yi, zi + 1);

                    if !hidden {
                        let cell = [low[0] + x as i32, low[1] + y as i32, low[2] + z as i32];
                        cells.push(SurfaceCell {
                            center: self.cell_center(cell),
                            ball,
                        });
                    }
                }
            }
        }

        cells
    }

    /// Cell index range that covers every ball.
    fn cell_bounds(&self) -> ([i32; 3], [i32; 3]) {
        let mut low = [i32::MAX; 3];
        let mut high = [i32::MIN; 3];

        for ball in &self.balls {
            let local = (ball.center - self.origin()) / self.cube_edge;
            let reach = ball.radius / self.cube_edge;

            for (axis, value) in [local.x, local.y, local.z].into_iter().enumerate() {
                low[axis] = low[axis].min((value - reach).floor() as i32);
                high[axis] = high[axis].max((value + reach).ceil() as i32);
            }
        }

        (low, high)
    }
}

/// Distance from `from` along `direction` to the point where the line leaves
/// a body of cubes for the last time: the top of its cubes in that direction.
/// The body is the union of `balls` (center and radius) on a grid of
/// `cube_edge` cubes with a corner at the first ball's center, like a
/// `VoxelBody`. `from`
/// is normally a point inside the body, like a ball center. Objects that
/// stood on a sphere stand here instead. Returns 0 when the line never
/// touches the body or the shape is invalid.
pub(super) fn surface_distance(
    balls: &[(Vec3, f32)],
    cube_edge: f32,
    from: Vec3,
    direction: Vec3,
) -> f32 {
    let Some(&(origin, _)) = balls.first() else {
        return 0.0;
    };
    if !cube_edge.is_finite() || cube_edge <= 0.0 || direction.length_squared() <= 0.0 {
        return 0.0;
    }

    let direction = direction.normalized();
    let inside = |point: Vec3| {
        let local = (point - origin) / cube_edge;
        let cell_center = origin
            + Vec3::new(
                local.x.floor() + 0.5,
                local.y.floor() + 0.5,
                local.z.floor() + 0.5,
            ) * cube_edge;

        balls
            .iter()
            .any(|&(center, radius)| (cell_center - center).length_squared() <= radius * radius)
    };
    let reach = balls
        .iter()
        .map(|&(center, radius)| (center - from).length() + radius)
        .fold(0.0, f32::max)
        + cube_edge * 2.0;
    let step = cube_edge / SURFACE_STEPS_PER_CUBE;
    let steps = (reach / step).ceil() as usize;
    // The cubes are a little smaller than their cells.
    let gap = cube_edge * (1.0 - VOXEL_CUBE_FILL) * 0.5;

    (0..=steps)
        .rev()
        .map(|index| index as f32 * step)
        .find(|&distance| inside(from + direction * distance))
        .map_or(0.0, |distance| (distance + step * 0.5 - gap).max(0.0))
}

/// `cube_edge`, or a smaller edge that fits `min_cubes_across` cubes in the
/// diameter of a ball of `radius`, so small rocks still look round.
pub(super) fn fitted_cube_edge(radius: f32, cube_edge: f32, min_cubes_across: f32) -> f32 {
    cube_edge.min(radius * 2.0 / min_cubes_across.max(1.0))
}

/// Adds the visible cubes of `body` to the scene, one after the other, and
/// then the dark cores of its opaque balls.
pub(super) fn add_voxel_body(
    scene: &mut Scene,
    body: &VoxelBody,
) -> Result<VoxelBodyParts, SpaceBuildError> {
    let cells = body.surface_cells();
    let (materials, palette_materials) = cell_materials(scene, body, &cells)?;
    let half = Vec3::new(body.cube_edge, body.cube_edge, body.cube_edge) * (0.5 * VOXEL_CUBE_FILL);
    let first_id = scene.object_count();

    for (cell, &material_id) in cells.iter().zip(&materials) {
        scene.add_cube(Cube::new(
            cell.center - half,
            cell.center + half,
            material_id,
        ))?;
    }

    let mut core_count = 0;
    for (index, ball) in body.balls.iter().enumerate() {
        let core_radius = ball.radius - body.cube_edge * CORE_DEPTH_CUBES;
        let opaque = scene
            .material(ball.material_id)
            .is_some_and(|material| material.transparency <= 0.0);

        if !opaque || core_radius <= 0.0 {
            continue;
        }

        // The core takes the darkest color painted by its ball.
        let core_material = cells
            .iter()
            .zip(&materials)
            .filter(|(cell, _)| cell.ball == index)
            .filter_map(|(_, &material_id)| {
                scene
                    .material(material_id)
                    .map(|material| (brightness(material.albedo), material_id))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map_or(ball.material_id, |(_, material_id)| material_id);

        scene.add_sphere(Sphere::new(ball.center, core_radius, core_material)?)?;
        core_count += 1;
    }

    Ok(VoxelBodyParts {
        first_id,
        cube_count: cells.len(),
        core_count,
        palette_materials,
    })
}

/// Material of every cell: the ball's own material when it has no texture,
/// or the nearest color of a palette built from the texture otherwise.
/// Returns the materials and how many palette materials were registered.
fn cell_materials(
    scene: &mut Scene,
    body: &VoxelBody,
    cells: &[SurfaceCell],
) -> Result<(Vec<usize>, usize), SpaceBuildError> {
    let mut materials: Vec<usize> = cells
        .iter()
        .map(|cell| body.balls[cell.ball].material_id)
        .collect();
    let mut registered = 0;
    let mut families: Vec<usize> = Vec::new();

    for ball in &body.balls {
        if !families.contains(&ball.material_id) {
            families.push(ball.material_id);
        }
    }

    for family in families {
        let Some(base) = scene.material(family).copied() else {
            continue;
        };
        if base.texture_id.is_none() {
            continue;
        }

        let members: Vec<usize> = (0..cells.len())
            .filter(|&index| materials[index] == family)
            .collect();
        let colors: Vec<Color> = members
            .iter()
            .map(|&index| {
                let cell = cells[index];
                let ball = body.balls[cell.ball];
                let direction = direction_or_up(cell.center - ball.center);

                base.albedo * material_texel(scene, base, Sphere::uv_from_normal(direction))
            })
            .collect();
        let palette = color_palette(&colors, VOXEL_PALETTE_SIZE);
        let mut palette_ids = Vec::with_capacity(palette.len());

        for color in &palette {
            palette_ids.push(scene.add_material(Material {
                albedo: color.clamped(),
                texture_id: None,
                ..base
            })?);
        }
        registered += palette_ids.len();

        for (&index, &color) in members.iter().zip(&colors) {
            materials[index] = palette_ids[nearest_color(&palette, color)];
        }
    }

    Ok((materials, registered))
}

fn direction_or_up(offset: Vec3) -> Vec3 {
    if offset.length_squared() > 1.0e-12 {
        offset.normalized()
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    }
}

/// Up to `size` representative colors (k-means), seeded with colors spread
/// by brightness so the result does not depend on the input order more than
/// needed. Duplicated colors are merged.
fn color_palette(colors: &[Color], size: usize) -> Vec<Color> {
    if colors.is_empty() || size == 0 {
        return Vec::new();
    }

    let mut by_brightness = colors.to_vec();
    by_brightness.sort_by(|left, right| brightness(*left).total_cmp(&brightness(*right)));
    let mut palette: Vec<Color> = (0..size.min(colors.len()))
        .map(|slot| {
            let position = (slot as f32 + 0.5) / size.min(colors.len()) as f32;
            by_brightness[((position * colors.len() as f32) as usize).min(colors.len() - 1)]
        })
        .collect();
    palette.dedup_by(|left, right| color_distance(*left, *right) < 1.0e-8);

    for _ in 0..PALETTE_ITERATIONS {
        let mut sums = vec![(Color::BLACK, 0usize); palette.len()];

        for &color in colors {
            let slot = nearest_color(&palette, color);
            sums[slot].0 += color;
            sums[slot].1 += 1;
        }
        for (entry, (sum, count)) in palette.iter_mut().zip(sums) {
            if count > 0 {
                *entry = sum * (1.0 / count as f32);
            }
        }
    }

    let mut merged: Vec<Color> = Vec::with_capacity(palette.len());
    for color in palette {
        if merged
            .iter()
            .all(|&other| color_distance(other, color) >= 1.0e-8)
        {
            merged.push(color);
        }
    }

    merged
}

fn nearest_color(palette: &[Color], color: Color) -> usize {
    palette
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| {
            color_distance(**left, color).total_cmp(&color_distance(**right, color))
        })
        .map_or(0, |(index, _)| index)
}

fn color_distance(left: Color, right: Color) -> f32 {
    let (r, g, b) = (left.r - right.r, left.g - right.g, left.b - right.b);

    r * r + g * g + b * b
}

fn brightness(color: Color) -> f32 {
    color.r * 0.299 + color.g * 0.587 + color.b * 0.114
}

impl From<VoxelError> for SpaceBuildError {
    fn from(error: VoxelError) -> Self {
        Self::Voxel(error)
    }
}

/// Checks that `parts` is a ball of equal cubes around `center`: every cell
/// lies inside `radius`, the cells reach its surface along every axis and
/// the grid has a cube corner at `center`. Returns the cube edge (the cell
/// size).
#[cfg(test)]
pub(crate) fn assert_voxel_ball(
    scene: &Scene,
    parts: VoxelBodyParts,
    center: Vec3,
    radius: f32,
) -> f32 {
    assert!(parts.cube_count > 0);
    let cubes: Vec<Cube> = parts
        .ids()
        .map(|id| *scene.objects()[id].as_cube().expect("voxel part is a cube"))
        .collect();
    let size = cubes[0].max.x - cubes[0].min.x;
    let edge = size / VOXEL_CUBE_FILL;
    let axes = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
    ];
    let mut reach = [f32::NEG_INFINITY; 6];

    for cube in &cubes {
        let extents = cube.max - cube.min;
        assert!((extents.x - size).abs() < 1.0e-4);
        assert!((extents.y - size).abs() < 1.0e-4);
        assert!((extents.z - size).abs() < 1.0e-4);

        let cell = (cube.min + cube.max) * 0.5;
        let offset = cell - center;
        assert!(offset.length() <= radius + 1.0e-4, "cell outside the ball");
        for component in [offset.x, offset.y, offset.z] {
            let steps = component / edge - 0.5;
            assert!((steps - steps.round()).abs() < 1.0e-2, "cell off the grid");
        }
        for (slot, axis) in reach.iter_mut().zip(axes) {
            *slot = slot.max(offset.dot(axis));
        }
    }
    for value in reach {
        assert!(
            value > radius - edge * 1.5,
            "cubes do not reach the surface"
        );
    }

    edge
}

#[cfg(test)]
mod tests {
    use super::{
        VOXEL_CUBE_FILL, VoxelBall, VoxelBody, VoxelError, add_voxel_body, assert_voxel_ball,
        color_palette, fitted_cube_edge, nearest_color, surface_distance,
    };
    use crate::{
        color::Color,
        material::Material,
        math::{Vec2, Vec3},
        ray::Ray,
        scene::Scene,
        space::SpaceBuildError,
        texture::{Texture, WrapMode},
    };

    fn scene_with(material: Material) -> (Scene, usize) {
        let mut scene = Scene::new();
        let id = scene.add_material(material).unwrap();

        (scene, id)
    }

    #[test]
    fn body_rejects_bad_edges_and_empty_ball_lists() {
        let ball = VoxelBall::new(Vec3::ZERO, 1.0, 0);

        for edge in [0.0, -0.1, f32::NAN, f32::INFINITY] {
            assert!(matches!(
                VoxelBody::new(&[ball], edge),
                Err(SpaceBuildError::Voxel(VoxelError::InvalidCubeEdge))
            ));
        }
        assert!(matches!(
            VoxelBody::new(&[], 0.1),
            Err(SpaceBuildError::Voxel(VoxelError::NoBalls))
        ));
        assert!(matches!(
            VoxelBody::new(&[VoxelBall::new(Vec3::ZERO, -1.0, 0)], 0.1),
            Err(SpaceBuildError::Sphere(_))
        ));
    }

    #[test]
    fn small_balls_get_smaller_cubes() {
        assert_eq!(fitted_cube_edge(2.0, 0.1, 7.0), 0.1);
        assert!((fitted_cube_edge(0.14, 0.1, 7.0) - 0.04).abs() < 1.0e-6);
    }

    #[test]
    fn only_cells_with_an_empty_neighbor_are_kept() {
        let body = VoxelBody::new(&[VoxelBall::new(Vec3::ZERO, 1.0, 0)], 0.1).unwrap();
        let cells = body.surface_cells();
        let solid = (-10..10)
            .flat_map(|x| (-10..10).flat_map(move |y| (-10..10).map(move |z| [x, y, z])))
            .filter(|&cell| body.contains_cell(cell))
            .count();

        assert!(!cells.is_empty());
        // A shell: far fewer cells than the solid ball, all near the surface.
        assert!(cells.len() * 3 < solid);
        for cell in cells {
            let distance = cell.center.length();
            assert!(distance <= 1.0);
            assert!(distance > 1.0 - 0.1 * 1.8);
        }
    }

    #[test]
    fn later_balls_paint_over_earlier_ones() {
        let body = VoxelBody::new(
            &[
                VoxelBall::new(Vec3::ZERO, 1.0, 0),
                VoxelBall::new(Vec3::new(0.0, 1.0, 0.0), 0.4, 1),
            ],
            0.1,
        )
        .unwrap();

        for cell in body.surface_cells() {
            let in_cap = (cell.center - Vec3::new(0.0, 1.0, 0.0)).length() <= 0.4;
            assert_eq!(cell.ball, usize::from(in_cap));
        }
    }

    #[test]
    fn surface_distance_finds_the_top_of_the_cubes() {
        let balls = [(Vec3::new(1.0, 2.0, 3.0), 1.0)];
        let edge = 0.1;
        let gap = edge * (1.0 - VOXEL_CUBE_FILL) * 0.5;
        let up = surface_distance(&balls, edge, balls[0].0, Vec3::new(0.0, 1.0, 0.0));

        // The top cells along +Y are centered at 0.95, so their top is at 1.0.
        assert!((up - (1.0 - gap)).abs() < edge * 0.05, "{up}");
        for direction in [
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(-0.3, 0.2, 0.9),
            Vec3::new(0.0, -1.0, 0.1),
        ] {
            let distance = surface_distance(&balls, edge, balls[0].0, direction);
            assert!((distance - 1.0).abs() < edge * 1.3, "{distance}");
        }
        assert_eq!(surface_distance(&[], edge, Vec3::ZERO, up_vector()), 0.0);
        assert_eq!(surface_distance(&balls, 0.0, Vec3::ZERO, up_vector()), 0.0);
    }

    fn up_vector() -> Vec3 {
        Vec3::new(0.0, 1.0, 0.0)
    }

    #[test]
    fn body_surface_distance_matches_the_free_function() {
        let body = VoxelBody::new(&[VoxelBall::new(Vec3::ZERO, 0.8, 3)], 0.07).unwrap();
        let direction = Vec3::new(0.2, 0.7, 0.4);

        assert_eq!(
            body.surface_distance(Vec3::ZERO, direction),
            surface_distance(&[(Vec3::ZERO, 0.8)], 0.07, Vec3::ZERO, direction)
        );
    }

    #[test]
    fn palette_keeps_distinct_colors_and_merges_repeats() {
        let dark = Color::new(0.1, 0.1, 0.1);
        let light = Color::new(0.9, 0.8, 0.7);
        let colors = [dark, dark, light, light, dark];
        let palette = color_palette(&colors, 8);

        let near = |left: Color, right: Color| {
            (left.r - right.r).abs() + (left.g - right.g).abs() + (left.b - right.b).abs() < 1.0e-5
        };

        assert_eq!(palette.len(), 2);
        assert!(near(palette[nearest_color(&palette, dark)], dark));
        assert!(near(palette[nearest_color(&palette, light)], light));
        assert!(color_palette(&[], 8).is_empty());
        let single = color_palette(&[light; 20], 8);
        assert_eq!(single.len(), 1);
        assert!(near(single[0], light));
    }

    #[test]
    fn opaque_textured_ball_becomes_flat_colored_cubes_with_a_core() {
        let mut scene = Scene::new();
        let texture = scene.add_texture(
            Texture::new(
                2,
                1,
                vec![Color::new(0.2, 0.2, 0.2), Color::new(0.8, 0.8, 0.8)],
            )
            .unwrap(),
        );
        let material = scene
            .add_material(Material::diffuse(Color::WHITE).with_texture(
                texture,
                Vec2::new(1.0, 1.0),
                WrapMode::Repeat,
            ))
            .unwrap();
        let body = VoxelBody::new(&[VoxelBall::new(Vec3::ZERO, 1.0, material)], 0.1).unwrap();
        let parts = add_voxel_body(&mut scene, &body).unwrap();

        assert_eq!(parts.first_id, 0);
        assert_eq!(parts.core_count, 1);
        assert_eq!(scene.object_count(), parts.total());
        assert!(parts.palette_materials >= 2 && parts.palette_materials <= 8);
        assert_eq!(scene.materials().len(), 1 + parts.palette_materials);
        assert!((assert_voxel_ball(&scene, parts, Vec3::ZERO, 1.0) - 0.1).abs() < 1.0e-5);
        for id in parts.ids() {
            let cube_material = scene.material(scene.objects()[id].material_id()).unwrap();
            assert!(cube_material.texture_id.is_none());
        }

        // The core is under the cubes: a ray from outside hits a cube, and
        // one down a seam between cubes hits the core instead of going
        // through the ball.
        let core_id = parts.total() - 1;
        let core = scene.objects()[core_id].as_sphere().unwrap();
        assert!(core.radius() < 1.0 - 0.1);
        scene.build_bvh();
        let cube_hit = scene
            .intersect(
                &Ray::new(Vec3::new(0.33, 0.17, 5.0), Vec3::new(0.0, 0.0, -1.0)),
                0.001,
                100.0,
            )
            .unwrap();
        assert!(cube_hit.position.z > 0.8);
        let seam_ray = Ray::new(Vec3::new(0.3, 0.2, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let seam_hit = scene.intersect(&seam_ray, 0.001, 100.0).unwrap();
        assert!(seam_hit.position.z > 0.0);
        assert_eq!(seam_hit.material_id, scene.objects()[core_id].material_id());
    }

    #[test]
    fn transparent_ball_stays_a_hollow_shell_with_its_own_material() {
        let glass = Material::new(Color::WHITE, 1.0, 100.0, 0.05, 0.9, 1.1, Color::BLACK);
        let (mut scene, material) = scene_with(glass);
        let body = VoxelBody::new(&[VoxelBall::new(Vec3::ZERO, 0.6, material)], 0.1).unwrap();
        let parts = add_voxel_body(&mut scene, &body).unwrap();

        assert_eq!(parts.core_count, 0);
        assert_eq!(parts.palette_materials, 0);
        assert!(
            parts
                .ids()
                .all(|id| scene.objects()[id].material_id() == material)
        );
        // Nothing near the center: what is inside the glass shows through.
        assert!(parts.ids().all(|id| {
            let cube = scene.objects()[id].as_cube().unwrap();
            ((cube.min + cube.max) * 0.5).length() > 0.3
        }));
    }
}
