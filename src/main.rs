mod framebuffer;
mod ray_intersect;
mod sphere;

use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};
use ray_intersect::{Intersect, RayIntersect, Vec3};
use sphere::Sphere;
use std::f32::consts::{PI, TAU};
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const MAX_BOUNCES: u32 = 5;
const RAY_BIAS: f32 = 0.001;
const FLOOR_Y: f32 = -1.0;
const BLACK_HOLE_Z: f32 = -0.15;
const LIGHT_POSITION: Vec3 = Vec3::new(-3.0, 4.5, 3.5);

#[derive(Clone, Copy)]
enum Material {
    Ceramic,
    BrushedMetal { color: Vec3, roughness: f32 },
    Glass { tint: Vec3, refractive_index: f32 },
}

struct SceneObject {
    sphere: Sphere,
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
            pitch: 0.32,
            distance: 4.2,
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
            self.pitch = 0.32;
        }

        self.pitch = self.pitch.clamp(-1.20, 1.30);
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

    let horizon = (1.0 - direction.y.abs() / 0.34).clamp(0.0, 1.0).powi(3);
    color = color + Vec3::new(0.42, 0.025, 0.30) * horizon;

    let longitude = 0.5 + direction.z.atan2(direction.x) / TAU;
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

    let latitude = 0.5 + direction.y.asin() / PI;
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

fn closest_sphere_hit(
    origin: Vec3,
    direction: Vec3,
    objects: &[SceneObject],
) -> Option<(usize, Intersect)> {
    objects
        .iter()
        .enumerate()
        .filter_map(|(index, object)| {
            object
                .sphere
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
        let Some(hit) = object.sphere.ray_intersect(origin, light_direction) else {
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

fn ceramic_texture(normal: Vec3) -> Vec3 {
    let u = 0.5 + normal.z.atan2(normal.x) / TAU;
    let v = 0.5 - normal.y.asin() / PI;
    let checker = ((u * 12.0).floor() as i32 + (v * 8.0).floor() as i32) & 1;

    if checker == 0 {
        Vec3::new(0.72, 0.055, 0.035)
    } else {
        Vec3::new(0.95, 0.78, 0.48)
    }
}

fn shade_black_hole(point: Vec3, objects: &[SceneObject]) -> Vec3 {
    let centered_x = point.x;
    let centered_z = point.z - BLACK_HOLE_Z;
    let radius = (centered_x * centered_x + centered_z * centered_z).sqrt();

    // Dentro del horizonte de eventos no escapa ninguna luz.
    if radius < 0.78 {
        return Vec3::ZERO;
    }

    let angle = centered_z.atan2(centered_x);
    let inner_fade = smoothstep(0.78, 1.08, radius);
    let outer_fade = 1.0 - smoothstep(3.6, 5.3, radius);
    let disk_mask = inner_fade * outer_fade;

    let spiral_phase = angle * 6.0 - radius * 10.5 + (angle * 3.0).sin() * 1.7;
    let hot_filament = (spiral_phase.sin() * 0.5 + 0.5).powi(7);
    let secondary_filament = ((spiral_phase * 0.47 + radius * 17.0).sin() * 0.5 + 0.5).powi(10);
    let turbulence = hash_2d((centered_x * 18.0).floor(), (centered_z * 18.0).floor());

    let heat = (1.0 - (radius - 0.78) / 4.6).clamp(0.0, 1.0);
    let cool_disk = Vec3::new(0.12, 0.025, 0.48);
    let hot_disk = Vec3::new(1.0, 0.12, 0.015);
    let disk_color = mix(cool_disk, hot_disk, heat);
    let visibility = shadow_visibility(point, Vec3::new(0.0, 1.0, 0.0), objects);
    let disk = disk_color
        * (disk_mask
            * (0.16 + hot_filament * 1.8 + secondary_filament * 0.7 + turbulence * 0.12)
            * (0.55 + visibility * 0.45));

    // Anillo de fotones intensamente brillante justo fuera del horizonte.
    let photon_ring = (1.0 - (radius - 0.86).abs() / 0.065)
        .clamp(0.0, 1.0)
        .powi(5);
    let lensing_halo = (1.0 - (radius - 0.98).abs() / 0.24).clamp(0.0, 1.0).powi(3);

    let distant_dust = hash_2d((centered_x * 7.0).floor(), (centered_z * 7.0).floor());
    let dust = smoothstep(0.975, 1.0, distant_dust) * outer_fade;

    Vec3::new(0.001, 0.002, 0.008)
        + disk
        + Vec3::new(1.0, 0.72, 0.32) * (photon_ring * 2.4)
        + Vec3::new(0.42, 0.04, 0.82) * (lensing_halo * 0.32)
        + Vec3::new(0.08, 0.48, 1.0) * (dust * 0.75)
}

fn trace_ray(origin: Vec3, direction: Vec3, objects: &[SceneObject], bounces_left: u32) -> Vec3 {
    if bounces_left == 0 {
        return sky_color(direction);
    }

    let sphere_hit = closest_sphere_hit(origin, direction, objects);
    let floor_hit = floor_intersect(origin, direction);

    if let Some((floor_distance, point)) = floor_hit
        && sphere_hit
            .as_ref()
            .is_none_or(|(_, hit)| floor_distance < hit.distance)
    {
        return shade_black_hole(point, objects);
    }

    if let Some((object_index, hit)) = sphere_hit {
        return shade_sphere(object_index, hit, direction, objects, bounces_left - 1);
    }

    sky_color(direction)
}

fn shade_sphere(
    object_index: usize,
    hit: Intersect,
    incoming: Vec3,
    objects: &[SceneObject],
    bounces_left: u32,
) -> Vec3 {
    let material = objects[object_index].material;
    let to_light = (LIGHT_POSITION - hit.point).normalized();
    let to_camera = -incoming;
    let rim = (1.0 - hit.normal.dot(to_camera).abs()).powi(3);

    match material {
        Material::Ceramic => {
            let base = ceramic_texture(hit.normal);
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
                + Vec3::new(0.38, 0.04, 0.72) * (rim * 0.14)
        }
        Material::BrushedMetal { color, roughness } => {
            let grain = ((hit.point.y * 95.0 + (hit.point.x * 14.0).sin() * 3.0).sin() * 0.5 + 0.5)
                * roughness;
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
                + Vec3::new(1.0, 0.08, 0.52) * (rim * 0.12)
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
                + Vec3::new(0.02, 0.82, 1.0) * (rim * 0.42)
        }
    }
}

fn build_scene() -> [SceneObject; 3] {
    [
        SceneObject {
            sphere: Sphere::new(Vec3::new(0.45, -0.15, 0.35), 0.72),
            material: Material::Glass {
                tint: Vec3::new(0.76, 0.94, 0.98),
                refractive_index: 1.52,
            },
        },
        SceneObject {
            sphere: Sphere::new(Vec3::new(-0.55, -0.15, -0.20), 0.82),
            material: Material::Ceramic,
        },
        SceneObject {
            sphere: Sphere::new(Vec3::new(0.45, 0.35, -0.55), 0.65),
            material: Material::BrushedMetal {
                color: Vec3::new(0.95, 0.55, 0.16),
                roughness: 0.16,
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
    let scale = (60.0_f32.to_radians() * 0.5).tan();

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
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, 0x000000);
    let objects = build_scene();
    let mut camera = Camera::new(Vec3::new(0.0, -0.05, -0.10));
    render(&mut framebuffer, &objects, &camera);

    let mut window = Window::new(
        "Esferas sobre agujero negro - WASD orbita - R reinicia - ESC sale",
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
        let camera = Camera::new(Vec3::new(0.0, -0.05, -0.10));
        let distance_to_target = (camera.position() - camera.target).length();

        assert!((distance_to_target - camera.distance).abs() < 0.0001);
    }

    #[test]
    fn ceramic_texture_alternates_colors() {
        let first = ceramic_texture(Vec3::new(1.0, 0.0, 0.0));
        let second = ceramic_texture(Vec3::new(0.866_025_4, 0.0, 0.5));

        assert_ne!(first, second);
    }

    #[test]
    fn event_horizon_is_darker_than_the_photon_ring() {
        let objects = build_scene();
        let center = shade_black_hole(Vec3::new(0.0, FLOOR_Y, BLACK_HOLE_Z), &objects);
        let ring = shade_black_hole(Vec3::new(0.86, FLOOR_Y, BLACK_HOLE_Z), &objects);

        assert!(ring.length() > center.length());
    }

    #[test]
    fn scene_contains_the_original_three_spheres() {
        let objects = build_scene();

        assert_eq!(objects.len(), 3);
        assert!((objects[0].sphere.radius - 0.72).abs() < 0.0001);
        assert!((objects[1].sphere.radius - 0.82).abs() < 0.0001);
        assert!((objects[2].sphere.radius - 0.65).abs() < 0.0001);
    }

    #[test]
    fn refraction_at_normal_incidence_keeps_direction() {
        let direction = Vec3::new(0.0, 0.0, -1.0);
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let refracted = refract(direction, normal, 1.0 / 1.5).unwrap();

        assert!((refracted - direction).length() < 0.0001);
    }
}
