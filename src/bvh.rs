use crate::{intersection::Intersection, math::Vec3, primitive::Primitive, ray::Ray};

const LEAF_SIZE: usize = 4;
const BOUNDS_PADDING: f32 = 0.0001;

#[derive(Debug, Clone, Copy)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    fn new(min: Vec3, max: Vec3) -> Option<Self> {
        let pad = |low: f32, high: f32| BOUNDS_PADDING.max(low.abs().max(high.abs()) * 0.000001);
        let x_pad = pad(min.x, max.x);
        let y_pad = pad(min.y, max.y);
        let z_pad = pad(min.z, max.z);
        let min = Vec3::new(min.x - x_pad, min.y - y_pad, min.z - z_pad);
        let max = Vec3::new(max.x + x_pad, max.y + y_pad, max.z + z_pad);
        let values = [min.x, min.y, min.z, max.x, max.y, max.z];
        values
            .iter()
            .all(|value| value.is_finite())
            .then_some(Self { min, max })
    }

    fn around(center: Vec3, extent: Vec3) -> Option<Self> {
        Self::new(center - extent, center + extent)
    }

    fn union(self, other: Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    fn centroid(self, axis: usize) -> f32 {
        let (min, max) = match axis {
            0 => (self.min.x, self.max.x),
            1 => (self.min.y, self.max.y),
            _ => (self.min.z, self.max.z),
        };
        min * 0.5 + max * 0.5
    }

    fn entry(self, ray: &Ray, t_min: f32, t_max: f32) -> Option<f32> {
        let mut near = t_min;
        let mut far = t_max;
        for (origin, direction, min, max) in [
            (ray.origin.x, ray.direction.x, self.min.x, self.max.x),
            (ray.origin.y, ray.direction.y, self.min.y, self.max.y),
            (ray.origin.z, ray.direction.z, self.min.z, self.max.z),
        ] {
            if direction.abs() <= BOUNDS_PADDING && origin >= min && origin <= max {
                continue;
            }
            if direction == 0.0 {
                if origin < min || origin > max {
                    return None;
                }
                continue;
            }
            let a = (min - origin) / direction;
            let b = (max - origin) / direction;
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far + BOUNDS_PADDING {
                return None;
            }
        }
        Some(near)
    }
}

fn oriented_extent(axes: [Vec3; 3], dimensions: Vec3) -> Vec3 {
    Vec3::new(
        axes[0].x.abs() * dimensions.x
            + axes[1].x.abs() * dimensions.y
            + axes[2].x.abs() * dimensions.z,
        axes[0].y.abs() * dimensions.x
            + axes[1].y.abs() * dimensions.y
            + axes[2].y.abs() * dimensions.z,
        axes[0].z.abs() * dimensions.x
            + axes[1].z.abs() * dimensions.y
            + axes[2].z.abs() * dimensions.z,
    ) + Vec3::new(BOUNDS_PADDING, BOUNDS_PADDING, BOUNDS_PADDING)
}

fn primitive_bounds(primitive: &Primitive) -> Option<Bounds> {
    match primitive {
        Primitive::Cube(shape) => Bounds::new(shape.min, shape.max),
        Primitive::Sphere(shape) => Bounds::around(
            shape.center(),
            Vec3::new(shape.radius(), shape.radius(), shape.radius()),
        ),
        Primitive::OrientedBox(shape) => {
            let basis = shape.orientation();
            Bounds::around(
                shape.center(),
                oriented_extent(
                    [basis.right(), basis.up(), basis.forward()],
                    shape.half_extents(),
                ),
            )
        }
        Primitive::Cylinder(shape) => {
            let basis = shape.orientation();
            let extent = oriented_extent(
                [basis.right(), basis.up(), basis.forward()],
                Vec3::new(shape.radius(), shape.half_height(), shape.radius()),
            );
            Bounds::around(shape.center(), extent)
        }
        Primitive::Cone(shape) => {
            let basis = shape.orientation();
            let extent = oriented_extent(
                [basis.right(), basis.up(), basis.forward()],
                Vec3::new(
                    shape.base_radius(),
                    shape.half_height(),
                    shape.base_radius(),
                ),
            );
            Bounds::around(shape.center(), extent)
        }
        Primitive::CurvedTetrahedron(shape) => {
            let radius = shape.circumradius() + BOUNDS_PADDING;
            Bounds::around(shape.center(), Vec3::new(radius, radius, radius))
        }
    }
}

#[derive(Debug)]
enum Node {
    Leaf {
        bounds: Bounds,
        indices: Vec<usize>,
    },
    Branch {
        bounds: Bounds,
        left: Box<Node>,
        right: Box<Node>,
    },
}

impl Node {
    fn bounds(&self) -> Bounds {
        match self {
            Self::Leaf { bounds, .. } | Self::Branch { bounds, .. } => *bounds,
        }
    }

    fn build(indices: &mut [usize], bounds: &[Option<Bounds>]) -> Self {
        // The builder only passes indices whose finite bounds were recorded.
        let mut combined = bounds[indices[0]].unwrap();
        for &index in &indices[1..] {
            combined = combined.union(bounds[index].unwrap());
        }
        if indices.len() <= LEAF_SIZE {
            return Self::Leaf {
                bounds: combined,
                indices: indices.to_vec(),
            };
        }
        let size = combined.max - combined.min;
        let axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };
        indices.sort_unstable_by(|a, b| {
            bounds[*a]
                .unwrap()
                .centroid(axis)
                .total_cmp(&bounds[*b].unwrap().centroid(axis))
                .then(a.cmp(b))
        });
        let (left, right) = indices.split_at_mut(indices.len() / 2);
        Self::Branch {
            bounds: combined,
            left: Box::new(Self::build(left, bounds)),
            right: Box::new(Self::build(right, bounds)),
        }
    }

    fn intersect(
        &self,
        objects: &[Primitive],
        ray: &Ray,
        t_min: f32,
        closest: &mut f32,
        winner: &mut Option<(usize, Intersection)>,
    ) {
        if self.bounds().entry(ray, t_min, *closest).is_none() {
            return;
        }
        match self {
            Self::Leaf { indices, .. } => {
                for &index in indices {
                    if let Some(hit) = objects[index].intersect(ray, t_min, *closest)
                        && winner.as_ref().is_none_or(|(old_index, _)| {
                            hit.distance < *closest
                                || (hit.distance == *closest && index > *old_index)
                        })
                    {
                        *closest = hit.distance;
                        *winner = Some((index, hit));
                    }
                }
            }
            Self::Branch { left, right, .. } => {
                let left_entry = left.bounds().entry(ray, t_min, *closest);
                let right_entry = right.bounds().entry(ray, t_min, *closest);
                match (left_entry, right_entry) {
                    (Some(a), Some(b)) if a <= b => {
                        left.intersect(objects, ray, t_min, closest, winner);
                        right.intersect(objects, ray, t_min, closest, winner);
                    }
                    (Some(_), Some(_)) => {
                        right.intersect(objects, ray, t_min, closest, winner);
                        left.intersect(objects, ray, t_min, closest, winner);
                    }
                    (Some(_), None) => left.intersect(objects, ray, t_min, closest, winner),
                    (None, Some(_)) => right.intersect(objects, ray, t_min, closest, winner),
                    (None, None) => {}
                }
            }
        }
    }

    fn intersects_any(&self, objects: &[Primitive], ray: &Ray, t_min: f32, t_max: f32) -> bool {
        if self.bounds().entry(ray, t_min, t_max).is_none() {
            return false;
        }
        match self {
            Self::Leaf { indices, .. } => indices
                .iter()
                .any(|&index| objects[index].intersect(ray, t_min, t_max).is_some()),
            Self::Branch { left, right, .. } => {
                left.intersects_any(objects, ray, t_min, t_max)
                    || right.intersects_any(objects, ray, t_min, t_max)
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Bvh {
    root: Option<Node>,
    unbounded: Vec<usize>,
}

impl Bvh {
    pub(crate) fn build(objects: &[Primitive]) -> Self {
        let bounds: Vec<_> = objects.iter().map(primitive_bounds).collect();
        let mut bounded = Vec::new();
        let mut unbounded = Vec::new();
        for (index, bound) in bounds.iter().enumerate() {
            if bound.is_some() {
                bounded.push(index);
            } else {
                unbounded.push(index);
            }
        }
        let root = (!bounded.is_empty()).then(|| Node::build(&mut bounded, &bounds));
        Self { root, unbounded }
    }

    pub(crate) fn intersect(
        &self,
        objects: &[Primitive],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<Intersection> {
        let mut closest = t_max;
        let mut winner = None;
        if let Some(root) = &self.root {
            root.intersect(objects, ray, t_min, &mut closest, &mut winner);
        }
        for &index in &self.unbounded {
            if let Some(hit) = objects[index].intersect(ray, t_min, closest)
                && winner.as_ref().is_none_or(|(old_index, _)| {
                    hit.distance < closest || (hit.distance == closest && index > *old_index)
                })
            {
                closest = hit.distance;
                winner = Some((index, hit));
            }
        }
        winner.map(|(_, hit)| hit)
    }

    pub(crate) fn intersects_any(
        &self,
        objects: &[Primitive],
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> bool {
        self.root
            .as_ref()
            .is_some_and(|root| root.intersects_any(objects, ray, t_min, t_max))
            || self
                .unbounded
                .iter()
                .any(|&index| objects[index].intersect(ray, t_min, t_max).is_some())
    }
}
