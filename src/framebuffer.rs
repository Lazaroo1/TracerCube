pub struct Framebuffer {
    pixels: Vec<u32>,
    clear_color: u32,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize, clear_color: u32) -> Self {
        Self {
            pixels: vec![clear_color; width * height],
            clear_color,
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(self.clear_color);
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_pixels_for_parallel_rendering() {
        let mut framebuffer = Framebuffer::new(3, 2, 0);
        framebuffer.pixels_mut()[4] = 0xff8040;

        assert_eq!(framebuffer.pixels()[4], 0xff8040);
    }
}
