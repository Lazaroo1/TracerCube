mod cube;
mod framebuffer;
mod ray_intersect;

use cube::Cube;
use framebuffer::Framebuffer;
use minifb::{Key, Window, WindowOptions};
use ray_intersect::{RayIntersect, Vec3};

const WIDTH: usize = 900;
const HEIGHT: usize = 650;

fn render(framebuffer: &mut Framebuffer, cube: &Cube) {
    let camera = Vec3::new(3.4, 2.7, 4.8);
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

            if let Some(hit) = cube.ray_intersect(camera, ray_direction) {
                // Lambert puro: no hay textura, luz ambiente ni brillo especular.
                let diffuse = hit.normal.dot(light_direction).max(0.0);
                let final_color = cube.color * diffuse;
                framebuffer.set_pixel(x, y, final_color.to_rgb());
            }
        }
    }
}

fn main() -> Result<(), minifb::Error> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, 0x0c111b);
    let cube = Cube::new(
        Vec3::new(-1.0, -1.0, -1.0),
        Vec3::new(1.0, 1.0, 1.0),
        Vec3::new(1.0, 0.32, 0.10),
    );

    render(&mut framebuffer, &cube);

    let mut window = Window::new(
        "TracerCube - ESC para salir",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(framebuffer.pixels(), WIDTH, HEIGHT)?;
    }

    Ok(())
}
