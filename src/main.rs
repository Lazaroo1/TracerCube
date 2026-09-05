mod cube;
mod framebuffer;
mod ray_intersect;

use cube::Cube;
use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};
use ray_intersect::{Intersect, RayIntersect, Vec3};
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MAX_BOUNCES: u32 = 5;
const RAY_BIAS: f32 = 0.001;
const FLOOR_Y: f32 = -0.82;
const LIGHT_POSITION: Vec3 = Vec3::new(-3.0, 4.5, 3.5);

#[derive(Clone, Copy)]
enum Material {
    Ceramic,
    BrushedMetal { color: Vec3, roughness: f32 },
    Glass { tint: Vec3, refractive_index: f32 },
}

struct SceneObject {
    cube: Cube,
    material: Material,
}

struct Camera {
    target: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl Camera {
    fn new(target: Vec3) -> Self {
        Self {
            target,
            yaw: 0.35,
            pitch: 0.35,
            distance: 8.0,
        }
    }

    fn position(&self) -> Vec3 {
        let horizontal_distance = self.distance * self.pitch.cos();
        self.target
            + Vec3::new(
                horizontal_distance * self.yaw.sin(),
                self.distance * self.pitch.sin(),
                horizontal_distance * self.yaw.cos(),
            )
    }

    fn update(&mut self, window: &Window, delta_seconds: f32) -> bool {
        let previous = (self.yaw, self.pitch);
        let rotation_step = 1.5 * delta_seconds;

        if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
            self.yaw -= rotation_step;
        }
        if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
            self.yaw += rotation_step;
        }
        if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
            self.pitch += rotation_step;
        }
        if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
            self.pitch -= rotation_step;
        }
        if window.is_key_down(Key::R) {
            self.yaw = 0.35;
            self.pitch = 0.35;
        }

        self.pitch = self.pitch.clamp(-1.35, 1.35);
        previous != (self.yaw, self.pitch)
    }
}

fn multiply(left: Vec3, right: Vec3) -> Vec3 {
    Vec3::new(left.x * right.x, left.y * right.y, left.z * right.z)
}

fn mix(start: Vec3, end: Vec3, amount: f32) -> Vec3 {
    start * (1.0 - amount) + end * amount
}

fn reflect(direction: Vec3, normal: Vec3) -> Vec3 {
    direction - normal * (2.0 * direction.dot(normal))
}

fn refract(direction: Vec3, normal: Vec3, eta_ratio: f32) -> Option<Vec3> {
    let cos_theta = (-direction).dot(normal).min(1.0);
    let perpendicular = (direction + normal * cos_theta) * eta_ratio;
    let parallel_length_squared = 1.0 - perpendicular.dot(perpendicular);

    if parallel_length_squared < 0.0 {
        None
    } else {
        Some((perpendicular - normal * parallel_length_squared.sqrt()).normalized())
    }
}

fn fresnel_schlick(cosine: f32, from_index: f32, to_index: f32) -> f32 {
    let base = ((from_index - to_index) / (from_index + to_index)).powi(2);
    base + (1.0 - base) * (1.0 - cosine).powi(5)
}

fn sky_color(direction: Vec3) -> Vec3 {
    let amount = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let sky = mix(
        Vec3::new(0.55, 0.68, 0.82),
        Vec3::new(0.055, 0.12, 0.28),
        amount,
    );
    let sun_direction = Vec3::new(-0.45, 0.72, 0.52).normalized();
    let sun = direction.dot(sun_direction).max(0.0).powf(320.0);
    sky + Vec3::new(1.0, 0.78, 0.5) * (sun * 2.5)
}

fn floor_intersect(origin: Vec3, direction: Vec3) -> Option<(f32, Vec3)> {
    if direction.y.abs() <= f32::EPSILON {
        return None;
    }

    let distance = (FLOOR_Y - origin.y) / direction.y;
    if distance <= RAY_BIAS {
        return None;
    }

    Some((distance, origin + direction * distance))
}

fn closest_cube_hit(
    origin: Vec3,
    direction: Vec3,
    objects: &[SceneObject],
) -> Option<(usize, Intersect)> {
    objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| {
            object
                .cube
                .ray_intersect(origin, direction)
                .map(|hit| (index, hit))
        })
        .min_by(|(_, left), (_, right)| left.distance.total_cmp(&right.distance))
}

fn shadow_visibility(point: Vec3, normal: Vec3, objects: &[SceneObject]) -> f32 {
    let to_light = LIGHT_POSITION - point;
    let light_distance = to_light.length();
    let light_direction = to_light * (1.0 / light_distance);
    let origin = point + normal * RAY_BIAS;
    let mut visibility = 1.0;

    for object in objects {
        let Some(hit) = object.cube.ray_intersect(origin, light_direction) else {
            continue;
        };
        if hit.distance >= light_distance {
            continue;
        }

        match object.material {
            Material::Glass { tint, .. } => {
                let transmission = (tint.x + tint.y + tint.z) / 3.0;
                visibility *= 0.65 * transmission;
            }
            _ => return 0.04,
        }
    }

    visibility
}

/// Proyección planar en cada cara para obtener coordenadas de textura continuas.
fn cube_uv(point: Vec3, normal: Vec3) -> (f32, f32) {
    if normal.x.abs() > 0.5 {
        (point.z, point.y)
    } else if normal.y.abs() > 0.5 {
        (point.x, point.z)
    } else {
        (point.x, point.y)
    }
}

fn ceramic_texture(point: Vec3, normal: Vec3) -> Vec3 {
    let (u, v) = cube_uv(point, normal);
    let checker = ((u * 5.0).floor() as i32 + (v * 5.0).floor() as i32) & 1;

    if checker == 0 {
        Vec3::new(0.72, 0.055, 0.035)
    } else {
        Vec3::new(0.95, 0.78, 0.48)
    }
}

fn shade_floor(point: Vec3, objects: &[SceneObject]) -> Vec3 {
    let normal = Vec3::new(0.0, 1.0, 0.0);
    let light_direction = (LIGHT_POSITION - point).normalized();
    let visibility = shadow_visibility(point, normal, objects);
    let checker = (point.x.floor() as i32 + point.z.floor() as i32) & 1;
    let base = if checker == 0 {
        Vec3::new(0.12, 0.14, 0.17)
    } else {
        Vec3::new(0.43, 0.46, 0.5)
    };
    let diffuse = normal.dot(light_direction).max(0.0) * visibility;

    base * (0.18 + diffuse * 0.82)
}

fn trace_ray(origin: Vec3, direction: Vec3, objects: &[SceneObject], bounces_left: u32) -> Vec3 {
    if bounces_left == 0 {
        return sky_color(direction);
    }

    let cube_hit = closest_cube_hit(origin, direction, objects);
    let floor_hit = floor_intersect(origin, direction);

    if let Some((floor_distance, point)) = floor_hit
        && cube_hit
            .as_ref()
            .is_none_or(|(_, hit)| floor_distance < hit.distance)
    {
        return shade_floor(point, objects);
    }

    if let Some((object_index, hit)) = cube_hit {
        return shade_cube(object_index, hit, direction, objects, bounces_left - 1);
    }

    sky_color(direction)
}

fn shade_cube(
    object_index: usize,
    hit: Intersect,
    incoming: Vec3,
    objects: &[SceneObject],
    bounces_left: u32,
) -> Vec3 {
    let material = objects[object_index].material;
    let to_light = (LIGHT_POSITION - hit.point).normalized();
    let to_camera = -incoming;

    match material {
        Material::Ceramic => {
            let base = ceramic_texture(hit.point, hit.normal);
            let visibility = shadow_visibility(hit.point, hit.normal, objects);
            let diffuse = hit.normal.dot(to_light).max(0.0) * visibility;
            let half_vector = (to_light + to_camera).normalized();
            let highlight = hit.normal.dot(half_vector).max(0.0).powf(80.0) * visibility;
            let reflected_direction = reflect(incoming, hit.normal).normalized();
            let reflected = trace_ray(
                hit.point + hit.normal * RAY_BIAS,
                reflected_direction,
                objects,
                bounces_left,
            );

            base * (0.14 + diffuse * 0.86)
                + Vec3::new(1.0, 0.95, 0.88) * (highlight * 0.35)
                + reflected * 0.045
        }
        Material::BrushedMetal { color, roughness } => {
            let (u, v) = cube_uv(hit.point, hit.normal);
            let grain = ((v * 95.0 + (u * 14.0).sin() * 3.0).sin() * 0.5 + 0.5) * roughness;
            let brushed_color = color * (0.82 + grain);
            let reflected_direction =
                (reflect(incoming, hit.normal) + hit.normal * grain).normalized();
            let reflected = trace_ray(
                hit.point + hit.normal * RAY_BIAS,
                reflected_direction,
                objects,
                bounces_left,
            );
            let visibility = shadow_visibility(hit.point, hit.normal, objects);
            let diffuse = hit.normal.dot(to_light).max(0.0) * visibility;
            let half_vector = (to_light + to_camera).normalized();
            let highlight = hit.normal.dot(half_vector).max(0.0).powf(140.0) * visibility;

            multiply(reflected, brushed_color) * 0.82
                + brushed_color * (0.04 + diffuse * 0.16)
                + Vec3::new(1.0, 0.82, 0.48) * (highlight * 0.7)
        }
        Material::Glass {
            tint,
            refractive_index,
        } => {
            let entering = incoming.dot(hit.normal) < 0.0;
            let normal = if entering { hit.normal } else { -hit.normal };
            let (from_index, to_index) = if entering {
                (1.0, refractive_index)
            } else {
                (refractive_index, 1.0)
            };
            let cosine = (-incoming).dot(normal).clamp(0.0, 1.0);
            let reflected_direction = reflect(incoming, normal).normalized();
            let reflected = trace_ray(
                hit.point + normal * RAY_BIAS,
                reflected_direction,
                objects,
                bounces_left,
            );

            let (transmitted, reflectance) = if let Some(refracted_direction) =
                refract(incoming, normal, from_index / to_index)
            {
                let refracted = trace_ray(
                    hit.point - normal * RAY_BIAS,
                    refracted_direction,
                    objects,
                    bounces_left,
                );
                (
                    multiply(refracted, tint),
                    fresnel_schlick(cosine, from_index, to_index),
                )
            } else {
                (Vec3::ZERO, 1.0)
            };

            let half_vector = (to_light + to_camera).normalized();
            let highlight = hit.normal.dot(half_vector).max(0.0).powf(220.0);

            transmitted * (1.0 - reflectance)
                + reflected * reflectance
                + Vec3::new(0.8, 0.95, 1.0) * (highlight * 0.9)
        }
    }
}

fn cube_at(center: Vec3, size: f32) -> Cube {
    let half = Vec3::new(size * 0.5, size * 0.5, size * 0.5);
    Cube::new(center - half, center + half)
}

fn build_scene() -> [SceneObject; 3] {
    const CUBE_SIZE: f32 = 1.3;

    [
        SceneObject {
            cube: cube_at(Vec3::new(-1.55, -0.15, 0.15), CUBE_SIZE),
            material: Material::Ceramic,
        },
        SceneObject {
            cube: cube_at(Vec3::new(0.0, -0.15, -0.10), CUBE_SIZE),
            material: Material::BrushedMetal {
                color: Vec3::new(0.95, 0.55, 0.16),
                roughness: 0.16,
            },
        },
        SceneObject {
            cube: cube_at(Vec3::new(1.55, -0.15, 0.15), CUBE_SIZE),
            material: Material::Glass {
                tint: Vec3::new(0.76, 0.94, 0.98),
                refractive_index: 1.52,
            },
        },
    ]
}

fn render(framebuffer: &mut Framebuffer, objects: &[SceneObject], camera: &Camera) {
    let camera_position = camera.position();
    let forward = (camera.target - camera_position).normalized();
    let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let up = right.cross(forward).normalized();
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let scale = (55.0_f32.to_radians() * 0.5).tan();

    framebuffer.clear();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_x = (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect_ratio * scale;
            let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * scale;
            let direction = (forward + right * screen_x + up * screen_y).normalized();
            let color = trace_ray(camera_position, direction, objects, MAX_BOUNCES);

            framebuffer.set_pixel(x, y, color.to_rgb());
        }
    }
}

fn main() -> Result<(), minifb::Error> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, 0x0c111b);
    let objects = build_scene();
    let mut camera = Camera::new(Vec3::new(0.0, -0.10, 0.05));
    render(&mut framebuffer, &objects, &camera);

    let mut window = Window::new(
        "Cubos texturizados - WASD orbita - R reinicia - ESC sale",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);
    let mut previous_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_seconds = now.duration_since(previous_frame).as_secs_f32().min(0.05);
        previous_frame = now;

        if camera.update(&window, delta_seconds) {
            render(&mut framebuffer, &objects, &camera);
        }

        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbital_camera_stays_at_the_requested_distance() {
        let camera = Camera::new(Vec3::new(0.0, -0.1, 0.05));
        let distance_to_target = (camera.position() - camera.target).length();

        assert!((distance_to_target - camera.distance).abs() < 0.0001);
    }

    #[test]
    fn ceramic_texture_alternates_colors() {
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let first = ceramic_texture(Vec3::new(0.01, 0.01, 0.65), normal);
        let second = ceramic_texture(Vec3::new(0.25, 0.01, 0.65), normal);

        assert_ne!(first, second);
    }

    #[test]
    fn all_three_cubes_have_the_same_size() {
        for object in build_scene() {
            let size = object.cube.max - object.cube.min;
            assert!((size.x - 1.3).abs() < 0.0001);
            assert!((size.y - 1.3).abs() < 0.0001);
            assert!((size.z - 1.3).abs() < 0.0001);
        }
    }

    #[test]
    fn refraction_at_normal_incidence_keeps_direction() {
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let refracted = refract(direction, normal, 1.0 / 1.5).unwrap();

        assert!((refracted - direction).length() < 0.0001);
    }
}
