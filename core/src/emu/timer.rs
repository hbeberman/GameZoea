use crate::emu::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

const DIV: u16 = 0xFF04;
const TIMA: u16 = 0xFF05;
const TMA: u16 = 0xFF06;
const TAC: u16 = 0xFF07;
const IF: u16 = 0xFF0F;

pub struct Timer {
    mem: Rc<RefCell<Memory>>,
    system_counter: u16,
    internal_tma: u8,
    prev_edge_signal: bool,
}

impl Timer {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Timer {
            mem,
            system_counter: 0xAC00,
            internal_tma: 0,
            prev_edge_signal: false,
        }
    }

    pub fn tick(&mut self, t: u128) {
        // Handle DIV reset FIRST
        let div_was_reset = self.check_write_div();
        if div_was_reset {
            self.system_counter = 0;
        }

        // Increment system counter
        if t.is_multiple_of(4) {
            let (result, _) = self.system_counter.overflowing_add(1);
            self.system_counter = result;
            self.mem_dbg_write(DIV, (result >> 8) as u8);
        }

        // Read TAC and calculate current edge signal AFTER incrementing
        let tac = self.mem_dbg_read(TAC);
        let timer_enabled = (tac & 0x4) != 0;
        let mask = match tac & 0x3 {
            0x0 => 1 << 9,
            0x1 => 1 << 1,
            0x2 => 1 << 5,
            0x3 => 1 << 7,
            _ => unreachable!(),
        };

        let curr_edge_signal = timer_enabled && (self.system_counter & mask) != 0;

        // If TAC was just written, reset edge detector to prevent spurious edge
        let tac_was_written = self.check_write_tac();
        if tac_was_written {
            self.prev_edge_signal = curr_edge_signal;
        }

        // Detect falling edge: previous tick had 1, current tick has 0
        // But don't detect if TAC was just written this tick
        let falling = !tac_was_written && self.prev_edge_signal && !curr_edge_signal;

        let mut tima = self.mem_dbg_read(TIMA);
        let tma = self.mem_dbg_read(TMA);

        if falling {
            let (result, overflow) = tima.overflowing_add(1);

            if overflow {
                self.mem_dbg_write(IF, self.mem_dbg_read(IF) | 0x4);
                tima = self.internal_tma;
            } else {
                tima = result;
            }
        }

        self.mem_dbg_write(TIMA, tima);

        // Update prev_edge_signal for next tick
        self.prev_edge_signal = curr_edge_signal;

        if t.is_multiple_of(4) {
            self.internal_tma = tma;
        }
    }

    fn with_mem_mut<R>(&self, f: impl FnOnce(&mut Memory) -> R) -> R {
        let mut mem = self.mem.borrow_mut();
        f(&mut mem)
    }

    fn with_mem<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        let mem = self.mem.borrow();
        f(&mem)
    }

    pub fn mem_read(&mut self) {
        self.with_mem_mut(|mem| {
            mem.read();
        });
    }

    pub fn mem_dbg_read(&self, addr: u16) -> u8 {
        self.with_mem(|mem| mem.dbg_read(addr))
    }

    pub fn mem_write(&mut self) {
        self.with_mem_mut(|mem| {
            mem.write();
        });
    }

    pub fn mem_dbg_write(&mut self, addr: u16, data: u8) {
        self.with_mem_mut(|mem| mem.dbg_write(addr, data));
    }

    pub fn check_write_div(&mut self) -> bool {
        self.with_mem_mut(|mem| mem.check_write_div())
    }

    pub fn check_write_tac(&mut self) -> bool {
        self.with_mem_mut(|mem| mem.check_write_tac())
    }
}
