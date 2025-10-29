use crate::app::window::*;

pub const BLACK: [u8; 4] = [0x29, 0x41, 0x39, 0xFF];
pub const DARK_GREY: [u8; 4] = [0x39, 0x59, 0x4A, 0xFF];
pub const LIGHT_GREY: [u8; 4] = [0x5A, 0x79, 0x42, 0xFF];
pub const WHITE: [u8; 4] = [0x7B, 0x82, 0x10, 0xFF];

pub struct Gameboy {
    cpu: Cpu,
    pixels: SharedPixels,
}

impl Gameboy {
    pub fn new(rom: &[u8], pixels: SharedPixels) -> Self {
        Gameboy {
            cpu: Cpu::init_dmg(rom),
            pixels,
        }
    }

    pub fn get_color(index: u8) -> [u8; 4] {
        match index {
            0x0 => BLACK,
            0x1 => DARK_GREY,
            0x2 => LIGHT_GREY,
            0x3 => WHITE,
            _ => unreachable!("invalid color value"),
        }
    }

    pub fn testwrite(&self, index: u8) {
        let Ok(mut guard) = self.pixels.lock() else {
            eprintln!("failed to lock shared pixels");
            return;
        };

        let Some(pixels) = guard.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();
        let mut i = index;
        for pixel in frame.chunks_exact_mut(4) {
            let color = Gameboy::get_color(i % 4);
            i = (i + 1) % 4;
            pixel.copy_from_slice(&color);
        }
    }

    pub fn run(&self) {
        let mut i: u8 = 0;
        loop {
            self.testwrite(i);
            i = (i + 1) % 4;

            std::thread::sleep(std::time::Duration::from_secs_f64(0.01));
        }
    }

    pub fn tick4(&mut self) {
        self.cpu.tick4();
    }
}
