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
    bg_fifo: Vec<Pixel>,
    obj_fifo: Vec<Pixel>,
    x: u8,
    y: u8,
    pub testing: usize,
}

pub struct Pixel {
    color: u8, // 0..=3
    palette: u8,
    // sprite_priority: u8, CGB only
    bg_priority: u8,
}

impl Ppu {
    pub fn headless_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Ppu {
            pixels: None,
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            y: 0,
            testing: 0,
        }
    }

    pub fn init_dmg(pixels: SharedPixels, mem: Rc<RefCell<Memory>>) -> Self {
        Ppu {
            pixels: Some(pixels),
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            y: 0,
            testing: 0,
        }
    }

    pub fn tick(&mut self, t: u128) {
        let _ = t;

        let color = ((self.x as usize + self.testing) % 4) as u8;

        let pixel = Pixel {
            color,
            palette: 0,
            bg_priority: 0,
        };
        self.bg_fifo.push(pixel);
        self.render();
    }

    pub fn render(&mut self) {
        let pixel = match self.bg_fifo.pop() {
            Some(pixel) => pixel,
            None => return,
        };

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

        let index = self.x as usize + self.y as usize * 160;
        if let Some(target) = pixels.frame_mut().get_mut((index * 4)..((index + 1) * 4)) {
            target.copy_from_slice(&Ppu::get_color(pixel.color));
        }

        self.x += 1;
        if self.x == 160 {
            self.x = 0;
            self.y += 1;
            if self.y == 144 {
                self.y = 0;
                _ = pixels.render();
            }
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
}
