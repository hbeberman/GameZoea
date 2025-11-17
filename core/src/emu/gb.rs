use crate::app::{
    control::{ControlMessage, ControlReceiver},
    window::*,
};
use crate::emu::cpu::{Cpu, CpuState};
use crate::emu::joypad::{Joypad, JoypadSnapshot};
use crate::emu::mem::Memory;
use crate::emu::ppu::*;
use crate::emu::regs::*;
use crate::emu::serial::{Serial, SerialState};
use crate::emu::timer::{Timer, TimerState};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

const NORMAL_CLOCK: f64 = 1.0 / 4_194_304.0;
const THROTTLE_BATCH_CYCLES: u32 = 8192;
const THROTTLE_SLEEP_GRACE_US: u64 = 100;

const L_CPU: u8 = 1 << 0;
const L_ADJ: u8 = 1 << 1;
const L_TIMER: u8 = 1 << 2;
const L_R: u8 = 1 << 3;
const L_MEM: u8 = 1 << 4;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub enum Comp {
    #[default]
    None,
    Cpu,
    Ppu,
    Timer,
    Serial,
    Joypad,
}

#[allow(dead_code)]
pub struct Gameboy {
    pub t: u64,
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub timer: Timer,
    pub serial: Serial,
    pub joypad: Joypad,
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
            joypad: Joypad::init_dmg(mem.clone()),
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
            joypad: Joypad::init_dmg(mem.clone()),
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
            joypad: Joypad::init_dmg(mem.clone()),
            mem,
        }
    }

    pub fn tick(&mut self, count: u64) {
        let mut remaining = count;
        while remaining > 0 {
            let cur = self.cpu.retired();

            self.with_mem_mut(|mem| mem.tick(self.t));
            self.timer.tick(self.t);
            self.cpu.tick(self.t);
            self.ppu.tick(self.t);
            self.serial.tick(self.t);
            self.joypad.tick(self.t);
            self.t += 1;
            remaining -= 1;
            if cur != self.cpu.retired() || (self.cpu.halted()) {
                //self.log_status(L_CPU + L_ADJ + L_R + L_TIMER);
                //                self.log_status(L_CPU);
                //self.log_status(L_CPU + L_TIMER + L_MEM);
            }
        }
    }

    pub fn step(&mut self, count: u64) {
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

    pub fn step_blargg(&mut self, count: u64, check: &str) {
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

    pub fn step_mooneye(&mut self, count: u64) {
        let mut i = count;
        let pass = [3, 5, 8, 13, 21, 34];
        let fail = [0x42; 6];
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        while i > 0 {
            let cur = self.cpu.retired();
            self.tick(1);
            if cur != self.cpu.retired() {
                i -= 1;
            }
            if self.serial.buf == pass {
                return;
            }
            if self.serial.buf == fail {
                let b = self.cpu.b();
                let d = self.cpu.d();
                let e = self.cpu.e();
                let oam0 = self.mem_dbg_read(0xFE00);
                let round1_oam = self.mem_dbg_read(0xFF84);
                let round1_b = self.mem_dbg_read(0xFF85);
                eprintln!(
                    "Mooneye fail: PC {:04X} B {:02X} D {:02X} E {:02X} OAM0 {:02X} R1B {:02X} R1OAM {:02X}",
                    self.cpu.cur_pc(),
                    b,
                    d,
                    e,
                    oam0,
                    round1_b,
                    round1_oam
                );
                panic!("Mooneye test failure!")
            }
            if start_time.elapsed() > timeout {
                panic!("Mooneye test timeout after 30 seconds!")
            }
        }
    }

    pub fn run(&mut self, control_rx: Option<ControlReceiver>, throttle_cycles: bool) {
        self.run_with_deadline(control_rx, None, throttle_cycles);
    }

    pub fn run_for(
        &mut self,
        control_rx: Option<ControlReceiver>,
        duration: Duration,
        throttle_cycles: bool,
    ) {
        self.run_with_deadline(control_rx, Some(duration), throttle_cycles);
    }

    fn run_with_deadline(
        &mut self,
        control_rx: Option<ControlReceiver>,
        limit: Option<Duration>,
        throttle_cycles: bool,
    ) {
        let normal_cycle = Duration::from_secs_f64(NORMAL_CLOCK);
        let throttle_batch = THROTTLE_BATCH_CYCLES as u64;
        let throttle_batch_duration = normal_cycle.mul_f64(THROTTLE_BATCH_CYCLES as f64);
        let _double_cycle = Duration::from_secs_f64(NORMAL_CLOCK / 2.0);
        let mut animate = Instant::now() + Duration::from_secs_f64(0.5);
        let stop_time = limit.map(|d| Instant::now() + d);
        let mut next_cycle_deadline = Instant::now();

        loop {
            if let Some(limit) = stop_time {
                if Instant::now() >= limit {
                    println!("{}", self.serial.buffmt());
                    return;
                }
            }

            if throttle_cycles {
                self.tick(throttle_batch);
                if let Some(limit) = stop_time {
                    if Instant::now() >= limit {
                        println!("{}", self.serial.buffmt());
                        return;
                    }
                }
                next_cycle_deadline += throttle_batch_duration;
                let now = Instant::now();
                if now < next_cycle_deadline {
                    let remaining = next_cycle_deadline - now;
                    let grace = Duration::from_micros(THROTTLE_SLEEP_GRACE_US);
                    if remaining > grace {
                        thread::sleep(remaining - grace);
                    }
                    while Instant::now() < next_cycle_deadline {
                        std::hint::spin_loop();
                    }
                } else {
                    next_cycle_deadline = now;
                }
            } else {
                self.tick(1);
            }
            if Instant::now() > animate {
                self.ppu.testing = self.ppu.testing.wrapping_add(1);
                animate = Instant::now() + Duration::from_secs_f64(1.0 / 30.0);
            }
            if let Some(rx) = control_rx.as_ref() {
                loop {
                    match rx.try_recv() {
                        Ok(ControlMessage::Exit) => {
                            println!("{}", self.serial.buffmt());
                            return;
                        }
                        Ok(ControlMessage::JoypadInput { button, pressed }) => {
                            self.joypad.enqueue_input(button, pressed);
                        }
                        Ok(ControlMessage::DumpState) => match self.dump_to_file() {
                            Ok(path) => {
                                println!("State dumped to {}", path.display());
                            }
                            Err(err) => {
                                eprintln!("Failed to dump state: {err}");
                            }
                        },
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => return,
                    }
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
                self.mem_dbg_read(DIV),
                self.mem_dbg_read(TIMA),
                self.mem_dbg_read(TMA),
                self.mem_dbg_read(TAC)
            )
        } else {
            String::new()
        };

        let memstr = if mem {
            let addr = [P1, IF, IE];
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

    fn with_mem_mut<R>(&self, f: impl FnOnce(&mut Memory) -> R) -> R {
        let mut mem = self.mem.borrow_mut();
        f(&mut mem)
    }

    pub fn save_state(&mut self) -> GameboyState {
        let memory = self.with_mem_mut(|mem| mem.snapshot());
        GameboyState {
            t: self.t,
            memory,
            cpu: self.cpu.save_state(),
            ppu: self.ppu.save_state(),
            timer: self.timer.save_state(),
            serial: self.serial.save_state(),
            joypad: self.joypad.save_state(),
        }
    }

    pub fn load_state(&mut self, state: GameboyState) {
        let GameboyState {
            t,
            memory,
            cpu,
            ppu,
            timer,
            serial,
            joypad,
        } = state;
        self.t = t;
        *self.mem.borrow_mut() = memory;
        self.timer.load_state(&timer);
        self.serial.load_state(&serial);
        self.joypad.load_state(&joypad);
        self.ppu.load_state(&ppu);
        self.cpu.load_state(&cpu);
    }

    pub fn dump_to_file(&mut self) -> io::Result<PathBuf> {
        let state = self.save_state();
        let data = serde_json::to_vec(&state)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let guid = Uuid::new_v4();
        let filename = format!("{guid}.core");
        let path = env::current_dir()?.join(filename);
        fs::write(&path, data)?;
        Ok(path)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameboyState {
    t: u64,
    memory: Memory,
    cpu: CpuState,
    ppu: PpuState,
    timer: TimerState,
    serial: SerialState,
    joypad: JoypadSnapshot,
}

impl GameboyState {
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let data = fs::read(path)?;
        // Try JSON first (new format), fall back to bincode (old format)
        serde_json::from_slice(&data)
            .or_else(|_| bincode::deserialize(&data))
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }

    pub fn cartridge_bytes(&self) -> &[u8] {
        self.memory.cartridge_data()
    }
}
