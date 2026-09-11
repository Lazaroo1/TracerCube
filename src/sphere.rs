use crate::ray_intersect::{Intersect, RayIntersect, Vec3};

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<Intersect> {
        let origin_to_center = origin - self.center;
        let a = direction.dot(direction);
        let half_b = origin_to_center.dot(direction);
        let c = origin_to_center.dot(origin_to_center) - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 || a <= f32::EPSILON {
            return None;
        }

        let discriminant_root = discriminant.sqrt();
        let mut distance = (-half_b - discriminant_root) / a;

        if distance <= f32::EPSILON {
            distance = (-half_b + discriminant_root) / a;
        }
        if distance <= f32::EPSILON {
            return None;
        }

        let point = origin + direction * distance;
        let normal = (point - self.center).normalized();

        Some(Intersect {
            distance,
            point,
            normal,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_through_the_center_hits_the_sphere() {
        let sphere = Sphere::new(Vec3::ZERO, 1.0);
        let hit = sphere
            .ray_intersect(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0))
            .expect("el rayo debería tocar la esfera");

        assert!((hit.distance - 2.0).abs() < 0.0001);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn ray_away_from_the_sphere_misses() {
        let sphere = Sphere::new(Vec3::ZERO, 1.0);

        assert!(
            sphere
                .ray_intersect(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 1.0, 0.0))
                .is_none()
        );
    }
}
