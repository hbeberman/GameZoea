use crate::app::window::{FrameSender, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::emu::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub const BLACK: [u8; 4] = [0x29, 0x41, 0x39, 0xFF];
pub const DARK_GREY: [u8; 4] = [0x39, 0x59, 0x4A, 0xFF];
pub const LIGHT_GREY: [u8; 4] = [0x5A, 0x79, 0x42, 0xFF];
pub const WHITE: [u8; 4] = [0x7B, 0x82, 0x10, 0xFF];

const FRAME_BYTES: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize) * 4;

#[allow(dead_code)]
pub struct Ppu {
    frame_tx: Option<FrameSender>,
    mem: Rc<RefCell<Memory>>,
    bg_fifo: Vec<Pixel>,
    obj_fifo: Vec<Pixel>,
    x: u8,
    y: u8,
    pub testing: usize,
    back_buffer: Vec<u8>,
}

#[allow(dead_code)]
pub struct Pixel {
    color: u8, // 0..=3
    palette: u8,
    // sprite_priority: u8, CGB only
    bg_priority: u8,
}

impl Ppu {
    pub fn headless_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Ppu {
            frame_tx: None,
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            y: 0,
            testing: 0,
            back_buffer: vec![0; FRAME_BYTES],
        }
    }

    pub fn init_dmg(frame_tx: FrameSender, mem: Rc<RefCell<Memory>>) -> Self {
        Ppu {
            frame_tx: Some(frame_tx),
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            y: 0,
            testing: 0,
            back_buffer: vec![0; FRAME_BYTES],
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

        let index = self.x as usize + self.y as usize * SCREEN_WIDTH as usize;
        if let Some(target) = self.back_buffer.get_mut((index * 4)..((index + 1) * 4)) {
            target.copy_from_slice(&Ppu::get_color(pixel.color));
        }

        let mut frame_complete = false;
        self.x += 1;
        if self.x == SCREEN_WIDTH as u8 {
            self.x = 0;
            self.y += 1;
            if self.y == SCREEN_HEIGHT as u8 {
                self.y = 0;
                frame_complete = true;
            }
        }

        if !frame_complete {
            return;
        }

        let Some(frame_tx) = &self.frame_tx else {
            return;
        };

        if let Err(err) = frame_tx.send(self.back_buffer.clone()) {
            eprintln!("failed to deliver frame: {err}");
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
