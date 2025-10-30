use crate::app::window::*;
use crate::emu::mem::Memory;
use crate::emu::ppu::*;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub struct Gameboy {
    pub t: u128,
    pub cpu: Cpu,
    pub ppu: Ppu,
    mem: Rc<RefCell<Memory>>,
}

impl Gameboy {
    pub fn cartless_dmg() -> Self {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::headless_dmg(mem.clone()),
            mem,
        }
    }

    pub fn headless_dmg(rom: &[u8]) -> Self {
        let mem = Rc::new(RefCell::new(Memory::new(rom)));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::headless_dmg(mem.clone()),
            mem,
        }
    }

    pub fn dmg(rom: &[u8], pixels: SharedPixels) -> Self {
        let mem = Rc::new(RefCell::new(Memory::new(rom)));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::init_dmg(pixels, mem.clone()),
            mem,
        }
    }

    pub fn tick(&mut self, count: u128) {
        for _ in 0..count {
            self.cpu.tick(self.t);
            self.ppu.tick(self.t);
            self.t += 1;
        }
    }

    pub fn run(&self) {
        let mut i: u8 = 0;
        loop {
            self.ppu.testwrite(i);
            i = (i + 1) % 4;
            std::thread::sleep(std::time::Duration::from_secs_f64(0.01));
        }
    }
}
