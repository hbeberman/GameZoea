use crate::app::window::*;
use crate::emu::mem::Memory;
use crate::emu::ppu::*;
use crate::emu::timer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

const NORMAL_CLOCK: f64 = 1.0 / 4_194_304.0;

#[allow(dead_code)]
pub struct Gameboy {
    pub t: u128,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub timer: Timer,
    mem: Rc<RefCell<Memory>>,
}

impl Gameboy {
    pub fn cartless_dmg() -> Self {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::headless_dmg(mem.clone()),
            timer: Timer::init_dmg(mem.clone()),
            mem,
        }
    }

    pub fn headless_dmg(rom: &[u8]) -> Self {
        let mem = Rc::new(RefCell::new(Memory::new(rom)));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::headless_dmg(mem.clone()),
            timer: Timer::init_dmg(mem.clone()),
            mem,
        }
    }

    pub fn dmg(rom: &[u8], frame_tx: FrameSender) -> Self {
        let mem = Rc::new(RefCell::new(Memory::new(rom)));
        Gameboy {
            t: 0,
            cpu: Cpu::init_dmg_with_memory(mem.clone()),
            ppu: Ppu::init_dmg(frame_tx, mem.clone()),
            timer: Timer::init_dmg(mem.clone()),
            mem,
        }
    }

    pub fn tick(&mut self, count: u128) {
        for _ in 0..count {
            self.cpu.tick(self.t);
            self.ppu.tick(self.t);
            self.timer.tick(self.t);
            self.t += 1;
        }
    }

    pub fn step(&mut self, count: u128) {
        let mut i = count;
        while i > 0 {
            let cur = self.cpu.retired();
            self.tick(1);
            if cur != self.cpu.retired() {
                i -= 1;
            }
        }
    }

    pub fn run(&mut self) {
        let normal_cycle = Duration::from_secs_f64(NORMAL_CLOCK);
        let _double_cycle = Duration::from_secs_f64(NORMAL_CLOCK / 2.0);
        let mut animate = Instant::now() + Duration::from_secs_f64(0.5);
        loop {
            self.tick(1);
            let target = Instant::now() + normal_cycle;
            while Instant::now() < target {}
            if Instant::now() > animate {
                self.ppu.testing = self.ppu.testing.wrapping_add(1);
                animate = Instant::now() + Duration::from_secs_f64(1.0 / 30.0);
            }
        }
    }
}
