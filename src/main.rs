mod cube;
mod framebuffer;
mod ray_intersect;

use cube::Cube;
use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};
use ray_intersect::{RayIntersect, Vec3};
use std::time::Instant;

const WIDTH: usize = 900;
const HEIGHT: usize = 650;

fn render(framebuffer: &mut Framebuffer, cubes: &[Cube]) {
    let camera = Vec3::new(4.2, 3.1, 6.5);
    let target = Vec3::ZERO;
    let camera_forward = (target - camera).normalized();
    let camera_right = camera_forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
    let camera_up = camera_right.cross(camera_forward).normalized();

    // Vector que va desde la superficie hacia una luz ubicada arriba y al frente.
    let light_direction = Vec3::new(0.45, 1.0, 0.8).normalized();
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let projection_scale = (50.0_f32.to_radians() * 0.5).tan();

    framebuffer.clear();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_x =
                (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect_ratio * projection_scale;
            let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * projection_scale;
            let ray_direction =
                (camera_forward + camera_right * screen_x + camera_up * screen_y).normalized();

            let mut nearest_distance = f32::INFINITY;
            let mut pixel_color = None;

            for cube in cubes {
                if let Some(hit) = cube.ray_intersect(camera, ray_direction)
                    && hit.distance < nearest_distance
                {
                    nearest_distance = hit.distance;

                    // Lambert puro: no hay textura, luz ambiente ni brillo especular.
                    let diffuse = hit.normal.dot(light_direction).max(0.0);
                    pixel_color = Some((cube.color * diffuse).to_rgb());
                }
            }

            if let Some(final_color) = pixel_color {
                framebuffer.set_pixel(x, y, final_color);
            }
        }
    }
}

fn main() -> Result<(), minifb::Error> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, 0x0c111b);
    let mut cubes = [
        Cube::new(
            Vec3::new(-1.7, -0.8, -0.8),
            Vec3::new(-0.1, 0.8, 0.8),
            Vec3::new(1.0, 0.32, 0.10),
        ),
        Cube::new(
            Vec3::new(0.2, -0.8, -0.35),
            Vec3::new(1.5, 0.5, 0.95),
            Vec3::new(0.08, 0.72, 1.0),
        ),
    ];

    let mut window = Window::new(
        "TracerCube - 1/2 selecciona - WASD mueve - ESC sale",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);
    let mut selected_cube = 0;
    let mut previous_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let delta_time = (now - previous_frame).as_secs_f32().min(0.05);
        previous_frame = now;

        if window.is_key_down(Key::Key1) {
            selected_cube = 0;
        } else if window.is_key_down(Key::Key2) {
            selected_cube = 1;
        }

        let movement_speed = 2.0 * delta_time;
        let mut movement = Vec3::ZERO;
        if window.is_key_down(Key::W) {
            movement.y += movement_speed;
        }
        if window.is_key_down(Key::S) {
            movement.y -= movement_speed;
        }
        if window.is_key_down(Key::A) {
            movement.x -= movement_speed;
        }
        if window.is_key_down(Key::D) {
            movement.x += movement_speed;
        }

        cubes[selected_cube].translate(movement);

        render(&mut framebuffer, &cubes);
        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}
