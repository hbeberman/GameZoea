use crate::app::{
    control::{ControlMessage, ControlReceiver},
    window::*,
};
use crate::emu::cpu::Cpu;
use crate::emu::mem::Memory;
use crate::emu::ppu::*;
use crate::emu::serial::Serial;
use crate::emu::timer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;
use std::time::{Duration, Instant};

const NORMAL_CLOCK: f64 = 1.0 / 4_194_304.0;

const L_CPU: u8 = 1 << 0;
const L_ADJ: u8 = 1 << 1;
const L_TIMER: u8 = 1 << 2;
const L_R: u8 = 1 << 3;
const L_MEM: u8 = 1 << 4;

#[derive(Debug, PartialEq, Clone)]
pub enum Comp {
    None,
    Cpu,
    Ppu,
    Timer,
    Serial,
}

#[allow(dead_code)]
pub struct Gameboy {
    pub t: u128,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub timer: Timer,
    pub serial: Serial,
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
            serial: Serial::init_dmg(mem.clone()),
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
            serial: Serial::init_dmg(mem.clone()),
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
            serial: Serial::init_dmg(mem.clone()),
            mem,
        }
    }

    pub fn tick(&mut self, count: u128) {
        for _ in 0..count {
            let cur = self.cpu.retired();
            self.timer.tick(self.t);
            self.cpu.tick(self.t);
            self.ppu.tick(self.t);
            self.serial.tick(self.t);
            self.t += 1;
            if cur != self.cpu.retired() || (self.cpu.halted()) {
                //self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
                //                self.log_status(L_CPU);
                //                self.log_status(L_CPU + L_TIMER + L_MEM);
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
                //self.log_status(L_CPU + L_TIMER);
                //self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
            }
        }
    }

    pub fn step_blargg(&mut self, count: u128, check: &str) {
        let mut i = count;
        let expected = format!("{}\n\n\nPassed\n", check);
        while i > 0 {
            let cur = self.cpu.retired();
            self.tick(1);
            if cur != self.cpu.retired() {
                i -= 1;
                //self.log_status(L_CPU + L_TIMER);
                //self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
            }

            if self.serial.buffmt() == expected {
                return;
            }
        }
    }

    pub fn run(&mut self, control_rx: Option<ControlReceiver>) {
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
            if let Some(rx) = control_rx.as_ref() {
                match rx.try_recv() {
                    Ok(ControlMessage::Exit) => {
                        println!("{}", self.serial.buffmt());
                        break;
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => break,
                }
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
        let mem = f & L_MEM != 0x00;
        if adj && self.cpu.prev_pc() == self.cpu.cur_pc() {
            return;
        }

        let pc = if adj {
            self.cpu.prev_pc()
        } else {
            self.cpu.cur_pc()
        };
        if pc == 0x0000 {
            return;
        }

        let regs_view = self.cpu.log_view(adj);

        let cpustr = if cpu {
            let pcmem = [
                self.mem_dbg_read(pc),
                self.mem_dbg_read(pc.wrapping_add(1)),
                self.mem_dbg_read(pc.wrapping_add(2)),
                self.mem_dbg_read(pc.wrapping_add(3)),
            ];
            format!(
                "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X} ",
                regs_view.a,
                regs_view.f,
                regs_view.b,
                regs_view.c,
                regs_view.d,
                regs_view.e,
                regs_view.h,
                regs_view.l,
                regs_view.sp,
                pc,
                pcmem[0],
                pcmem[1],
                pcmem[2],
                pcmem[3],
            )
        } else {
            String::new()
        };

        let retiredstr = if retired {
            let retired = self.cpu.retired().saturating_sub(2);
            format!("|| R:{:04X} ", retired)
        } else {
            String::new()
        };

        let timerstr = if timer {
            format!(
                "|| DIV:{:02X} TIMA:{:02X} TMA:{:02X} TAC:{:02X} ",
                self.mem_dbg_read(0xFF04),
                self.mem_dbg_read(0xFF05),
                self.mem_dbg_read(0xFF06),
                self.mem_dbg_read(0xFF07)
            )
        } else {
            String::new()
        };

        let memstr = if mem {
            let addr = [0xFFFF, 0xFF0F];
            let s = format!(
                "||{}",
                addr.iter()
                    .map(|&a| format!(" {:04X}:{:02X}", a, self.mem_dbg_read(a)))
                    .collect::<String>()
            );
            s
        } else {
            String::new()
        };

        println!("{}{}{}{}", cpustr, retiredstr, timerstr, memstr);
    }

    fn with_mem<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        let mem = self.mem.borrow();
        f(&mem)
    }

    pub fn mem_dbg_read(&self, addr: u16) -> u8 {
        self.with_mem(|mem| mem.dbg_read(addr))
    }
}
