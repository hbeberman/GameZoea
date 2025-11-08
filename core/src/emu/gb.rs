use crate::app::window::*;
use crate::emu::mem::Memory;
use crate::emu::ppu::*;
use crate::emu::timer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

const NORMAL_CLOCK: f64 = 1.0 / 4_194_304.0;

const L_CPU: u8 = 1 << 0;
const L_ADJ: u8 = 1 << 1;
const L_TIMER: u8 = 1 << 2;
const L_R: u8 = 1 << 3;

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
            let cur = self.cpu.retired();
            self.cpu.tick(self.t);
            self.ppu.tick(self.t);
            self.timer.tick(self.t);
            self.t += 1;
            if cur != self.cpu.retired() {
                self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
            }
        }
    }

    pub fn step(&mut self, count: u128) {
        let mut i = count;
        while i > 0 {
            let cur = self.cpu.retired();
            self.tick(1);
            if cur != self.cpu.retired() {
                i -= 1;
                self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
            }
            if self.cpu.halted() {
                self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
                return;
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

    #[allow(dead_code)]
    fn log_status_all(&self) {
        self.log_status(0xFF);
    }

    fn log_status(&self, f: u8) {
        let cpu = f & L_CPU != 0x00;
        let adj = f & L_ADJ != 0x00;
        let timer = f & L_TIMER != 0x00;
        let retired = f & L_R != 0x00;
        let pc = if adj {
            self.cpu.prev_pc()
        } else {
            self.cpu.pc()
        };
        let cpustr = if cpu {
            &format!(
                "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X} ",
                self.cpu.a(),
                self.cpu.f(),
                self.cpu.b(),
                self.cpu.c(),
                self.cpu.d(),
                self.cpu.e(),
                self.cpu.h(),
                self.cpu.l(),
                self.cpu.sp(),
                pc,
                self.mem_dbg_read(self.cpu.pc()),
                self.mem_dbg_read(self.cpu.pc() + 1),
                self.mem_dbg_read(self.cpu.pc() + 2),
                self.mem_dbg_read(self.cpu.pc() + 3),
            )
        } else {
            ""
        };

        let retiredstr = if retired {
            let retired = self.cpu.retired().saturating_sub(2);
            &format!("|| R:{:04X} ", retired)
        } else {
            ""
        };

        let timerstr = if timer {
            &format!(
                "|| DIV:{:02X} TIMA:{:02X} TMA:{:02X} TAC:{:02X} ",
                self.mem_dbg_read(0xFF04),
                self.mem_dbg_read(0xFF05),
                self.mem_dbg_read(0xFF06),
                self.mem_dbg_read(0xFF07)
            )
        } else {
            ""
        };

        if pc != 0x0000 {
            println!("{}{}{}", cpustr, retiredstr, timerstr);
        };
    }

    fn with_mem<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        let mem = self.mem.borrow();
        f(&mem)
    }

    pub fn mem_dbg_read(&self, addr: u16) -> u8 {
        self.with_mem(|mem| mem.dbg_read(addr))
    }
}
