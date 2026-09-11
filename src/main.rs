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

fn smoothstep(edge_start: f32, edge_end: f32, value: f32) -> f32 {
    let amount = ((value - edge_start) / (edge_end - edge_start)).clamp(0.0, 1.0);
    amount * amount * (3.0 - 2.0 * amount)
}

fn hash_2d(x: f32, y: f32) -> f32 {
    let value = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    value - value.floor()
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
    let vertical = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let mut color = mix(
        Vec3::new(0.035, 0.008, 0.085),
        Vec3::new(0.002, 0.012, 0.055),
        vertical,
    );

    // Resplandor magenta en el horizonte.
    let horizon = (1.0 - direction.y.abs() / 0.34).clamp(0.0, 1.0).powi(3);
    color = color + Vec3::new(0.42, 0.025, 0.30) * horizon;

    // Nebulosa ondulada que cruza el cielo.
    let longitude = 0.5 + direction.z.atan2(direction.x) / std::f32::consts::TAU;
    let nebula_center = 0.16 + (longitude * 15.0).sin() * 0.09;
    let nebula_distance = (direction.y - nebula_center).abs();
    let nebula_band = (1.0 - nebula_distance / 0.25).clamp(0.0, 1.0).powi(3);
    let nebula_detail = ((longitude * 83.0 + direction.y * 37.0).sin() * 0.5 + 0.5).powi(2);
    let nebula_color = mix(
        Vec3::new(0.17, 0.03, 0.52),
        Vec3::new(0.02, 0.58, 0.78),
        nebula_detail,
    );
    color = color + nebula_color * (nebula_band * (0.28 + nebula_detail * 0.55));

    // Campo de estrellas estable en coordenadas esféricas.
    let latitude = 0.5 + direction.y.asin() / std::f32::consts::PI;
    let star_x = longitude * 420.0;
    let star_y = latitude * 210.0;
    let cell_x = star_x.floor();
    let cell_y = star_y.floor();
    let local_x = star_x - cell_x - 0.5;
    let local_y = star_y - cell_y - 0.5;
    let star_seed = hash_2d(cell_x, cell_y);
    let star_core = 1.0 - smoothstep(0.02, 0.34, (local_x * local_x + local_y * local_y).sqrt());
    let star_brightness = smoothstep(0.982, 1.0, star_seed) * star_core.powi(4);
    let star_tint = mix(
        Vec3::new(0.45, 0.75, 1.0),
        Vec3::new(1.0, 0.55, 0.92),
        hash_2d(cell_y, cell_x),
    );
    color = color + star_tint * (star_brightness * 2.6);

    // Planeta violeta con atmósfera y anillo inclinado.
    let planet_direction = Vec3::new(-0.52, 0.34, -0.78).normalized();
    let planet_right = Vec3::new(0.0, 1.0, 0.0)
        .cross(planet_direction)
        .normalized();
    let planet_up = planet_direction.cross(planet_right).normalized();
    let planet_depth = direction.dot(planet_direction);
    let planet_x = direction.dot(planet_right);
    let planet_y = direction.dot(planet_up);
    let planet_radius = (planet_x * planet_x + planet_y * planet_y).sqrt();
    let atmosphere = smoothstep(0.16, 0.105, planet_radius) * smoothstep(0.65, 0.95, planet_depth);
    color = color + Vec3::new(0.20, 0.35, 1.0) * (atmosphere * 0.75);

    let ring_radius = ((planet_x / 0.205).powi(2) + (planet_y / 0.050).powi(2)).sqrt();
    let ring = (1.0 - (ring_radius - 1.0).abs() / 0.16)
        .clamp(0.0, 1.0)
        .powi(2)
        * smoothstep(0.65, 0.95, planet_depth);
    color = color + Vec3::new(1.0, 0.18, 0.72) * (ring * 1.35);

    let planet_mask =
        smoothstep(0.112, 0.102, planet_radius) * smoothstep(0.65, 0.95, planet_depth);
    let surface_light =
        (0.24 + (-planet_x * 5.0 + planet_y * 2.0).clamp(-0.1, 0.76)).clamp(0.12, 1.0);
    let surface_bands = (planet_y * 110.0 + (planet_x * 55.0).sin()).sin() * 0.08 + 0.92;
    let planet_color = Vec3::new(0.48, 0.055, 0.72) * (surface_light * surface_bands);
    mix(color, planet_color, planet_mask)
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

fn cube_edge_glow(cube: &Cube, point: Vec3, normal: Vec3) -> f32 {
    let edge_distance = if normal.x.abs() > 0.5 {
        (point.y - cube.min.y)
            .min(cube.max.y - point.y)
            .min(point.z - cube.min.z)
            .min(cube.max.z - point.z)
    } else if normal.y.abs() > 0.5 {
        (point.x - cube.min.x)
            .min(cube.max.x - point.x)
            .min(point.z - cube.min.z)
            .min(cube.max.z - point.z)
    } else {
        (point.x - cube.min.x)
            .min(cube.max.x - point.x)
            .min(point.y - cube.min.y)
            .min(cube.max.y - point.y)
    };

    (1.0 - edge_distance / 0.055).clamp(0.0, 1.0).powi(3)
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
        Vec3::new(0.015, 0.035, 0.10)
    } else {
        Vec3::new(0.21, 0.018, 0.25)
    };
    let diffuse = normal.dot(light_direction).max(0.0) * visibility;
    let distance = (point.x * point.x + point.z * point.z).sqrt();
    let distance_fade = 1.0 / (1.0 + distance * distance * 0.018);

    let fraction_x = point.x - point.x.floor();
    let fraction_z = point.z - point.z.floor();
    let grid_distance = fraction_x
        .min(1.0 - fraction_x)
        .min(fraction_z.min(1.0 - fraction_z));
    let grid_glow = (1.0 - grid_distance / 0.045).clamp(0.0, 1.0).powi(3);
    let grid_color = if checker == 0 {
        Vec3::new(0.02, 0.85, 1.0)
    } else {
        Vec3::new(1.0, 0.04, 0.72)
    };

    // Tres halos sobre el tablero conectan visualmente el suelo con los cubos.
    let pool = |center_x: f32, center_z: f32| {
        let dx = point.x - center_x;
        let dz = point.z - center_z;
        (1.0 - (dx * dx + dz * dz).sqrt() / 1.45)
            .clamp(0.0, 1.0)
            .powi(3)
    };
    let floor_halos = Vec3::new(1.0, 0.08, 0.03) * pool(-1.55, 0.15)
        + Vec3::new(1.0, 0.25, 0.02) * pool(0.0, -0.10)
        + Vec3::new(0.02, 0.70, 1.0) * pool(1.55, 0.15);

    base * (0.12 + diffuse * 0.88)
        + grid_color * (grid_glow * distance_fade * 1.15)
        + floor_halos * (0.20 * distance_fade)
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
    let object = &objects[object_index];
    let material = object.material;
    let to_light = (LIGHT_POSITION - hit.point).normalized();
    let to_camera = -incoming;
    let edge = cube_edge_glow(&object.cube, hit.point, hit.normal);
    let rim = (1.0 - hit.normal.dot(to_camera).abs()).powi(3);

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
                + Vec3::new(1.0, 0.12, 0.025) * (edge * 1.2)
                + Vec3::new(0.08, 0.50, 1.0) * (rim * 0.08)
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
                + Vec3::new(1.0, 0.25, 0.035) * (edge * 1.45)
                + Vec3::new(0.95, 0.04, 0.55) * (rim * 0.12)
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
                + Vec3::new(0.02, 0.88, 1.0) * (edge * 1.8 + rim * 0.42)
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
    fn cube_edges_glow_more_than_the_face_center() {
        let cube = cube_at(Vec3::ZERO, 2.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let center_glow = cube_edge_glow(&cube, Vec3::new(0.0, 0.0, 1.0), normal);
        let edge_glow = cube_edge_glow(&cube, Vec3::new(0.99, 0.0, 1.0), normal);

        assert!(edge_glow > center_glow);
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
