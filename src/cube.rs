use crate::ray_intersect::{Intersect, RayIntersect, Vec3};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub color: Vec3,
}

impl Cube {
    pub const fn new(min: Vec3, max: Vec3, color: Vec3) -> Self {
        Self { min, max, color }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<Intersect> {
        let slabs = [
            (
                origin.x,
                direction.x,
                self.min.x,
                self.max.x,
                Vec3::new(-1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ),
            (
                origin.y,
                direction.y,
                self.min.y,
                self.max.y,
                Vec3::new(0.0, -1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ),
            (
                origin.z,
                direction.z,
                self.min.z,
                self.max.z,
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.0, 0.0, 1.0),
            ),
        ];

        let mut nearest = f32::NEG_INFINITY;
        let mut farthest = f32::INFINITY;
        let mut near_normal = Vec3::ZERO;
        let mut far_normal = Vec3::ZERO;

        for (ray_origin, ray_direction, min, max, min_normal, max_normal) in slabs {
            if ray_direction.abs() <= f32::EPSILON {
                if ray_origin < min || ray_origin > max {
                    return None;
                }
                continue;
            }

            let min_distance = (min - ray_origin) / ray_direction;
            let max_distance = (max - ray_origin) / ray_direction;
            let (axis_near, axis_far, axis_near_normal, axis_far_normal) =
                if min_distance <= max_distance {
                    (min_distance, max_distance, min_normal, max_normal)
                } else {
                    (max_distance, min_distance, max_normal, min_normal)
                };

            if axis_near > nearest {
                nearest = axis_near;
                near_normal = axis_near_normal;
            }
            if axis_far < farthest {
                farthest = axis_far;
                far_normal = axis_far_normal;
            }
            if nearest > farthest {
                return None;
            }
        }

        let (distance, normal) = if nearest > f32::EPSILON {
            (nearest, near_normal)
        } else if farthest > f32::EPSILON {
            // Este caso permite lanzar rayos desde el interior del cubo.
            (farthest, far_normal)
        } else {
            return None;
        };

        Some(Intersect {
            distance,
            point: origin + direction * distance,
            normal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cube() -> Cube {
        Cube::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.2, 0.1),
        )
    }

    #[test]
    fn ray_hits_front_face() {
        let hit = test_cube()
            .ray_intersect(Vec3::new(0.0, 0.0, 4.0), Vec3::new(0.0, 0.0, -1.0))
            .expect("el rayo debe tocar el cubo");

        assert!((hit.distance - 3.0).abs() < 0.0001);
        assert_eq!(hit.point, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn ray_misses_cube() {
        assert!(
            test_cube()
                .ray_intersect(Vec3::new(3.0, 3.0, 4.0), Vec3::new(0.0, 0.0, -1.0))
                .is_none()
        );
    }

    #[test]
    fn ray_can_exit_from_inside() {
        let hit = test_cube()
            .ray_intersect(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0))
            .expect("el rayo debe salir del cubo");

        assert_eq!(hit.point, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(hit.normal, Vec3::new(1.0, 0.0, 0.0));
    }
}
