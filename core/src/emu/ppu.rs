use crate::app::window::SharedPixels;
use crate::emu::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub const BLACK: [u8; 4] = [0x29, 0x41, 0x39, 0xFF];
pub const DARK_GREY: [u8; 4] = [0x39, 0x59, 0x4A, 0xFF];
pub const LIGHT_GREY: [u8; 4] = [0x5A, 0x79, 0x42, 0xFF];
pub const WHITE: [u8; 4] = [0x7B, 0x82, 0x10, 0xFF];

#[allow(dead_code)]
pub struct Ppu {
    pixels: Option<SharedPixels>,
    mem: Rc<RefCell<Memory>>,
}

impl Ppu {
    pub fn headless_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Ppu { pixels: None, mem }
    }

    pub fn init_dmg(pixels: SharedPixels, mem: Rc<RefCell<Memory>>) -> Self {
        Ppu {
            pixels: Some(pixels),
            mem,
        }
    }

    pub fn tick(&mut self, t: u128) {
        let _ = t;
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
        let pixels = match &self.pixels {
            Some(pixels) => pixels,
            None => return,
        };
        let Ok(mut guard) = pixels.lock() else {
            eprintln!("failed to lock shared pixels");
            return;
        };

        let Some(pixels) = guard.as_mut() else {
            return;
        };

        let frame = pixels.frame_mut();
        let mut i = index;
        for pixel in frame.chunks_exact_mut(4) {
            let color = Ppu::get_color(i % 4);
            i = (i + 1) % 4;
            pixel.copy_from_slice(&color);
        }
    }
}
