pub struct Framebuffer {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    clear_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, clear_color: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![clear_color; width * height],
            clear_color,
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(self.clear_color);
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_a_pixel_in_row_major_order() {
        let mut framebuffer = Framebuffer::new(3, 2, 0);
        framebuffer.set_pixel(1, 1, 0xff8040);

        assert_eq!(framebuffer.pixels()[4], 0xff8040);
    }
}
