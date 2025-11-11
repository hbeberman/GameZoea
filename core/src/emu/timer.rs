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
    prev_signal: bool,
    last_tac: u8,
    overflow_delay: u8,
}

impl Timer {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        let tima = 0x00;
        let internal_tma = 0x00;
        let last_tac = 0xF8;

        let mut timer = Timer {
            mem,
            system_counter: 0,
            internal_tma,
            prev_signal: false,
            last_tac,
            overflow_delay: 0,
        };

        timer.mem_write(TIMA, tima);
        timer.mem_write(TMA, internal_tma);
        timer.mem_write(TAC, last_tac);
        timer
    }

    fn timer_bit_mask(tac: u8) -> u16 {
        match tac & 0x3 {
            0x0 => 1 << 7,
            0x1 => 1 << 1,
            0x2 => 1 << 3,
            0x3 => 1 << 5,
            _ => unreachable!(),
        }
    }

    fn timer_signal(&self, counter: u16, tac: u8) -> bool {
        if tac & 0x4 == 0 {
            return false;
        }
        (counter & Self::timer_bit_mask(tac)) != 0
    }

    pub fn tick(&mut self, t: u128) {
        self.internal_tma = self.mem_read(TMA);

        if self.overflow_delay > 0 {
            self.overflow_delay -= 1;
            if self.overflow_delay == 0 {
                self.mem_write(TIMA, self.internal_tma);
                self.mem_write(IF, self.mem_read(IF) | 0x4);
            }
        }

        let mut prev_signal = self.prev_signal;
        let mut skip_counter_tick = false;

        if self.check_write_div() {
            let tac = self.mem_read(TAC);
            let signal_before = self.timer_signal(self.system_counter, tac);
            self.system_counter = 0;
            self.mem_write(DIV, 0);
            let signal_after = self.timer_signal(self.system_counter, tac);

            if signal_before && !signal_after {
                self.increment_tima();
            }
            prev_signal = signal_after;
            skip_counter_tick = true;
        }

        if self.check_write_tac() {
            let old_tac = self.last_tac;
            let new_tac = self.mem_read(TAC);
            self.last_tac = new_tac;

            let signal_before = self.timer_signal(self.system_counter, old_tac);
            let signal_after = self.timer_signal(self.system_counter, new_tac);

            if signal_before && !signal_after {
                self.increment_tima();
            }
            prev_signal = signal_after;
        }

        if t.is_multiple_of(4) && !skip_counter_tick {
            self.system_counter = (self.system_counter + 1) & 0x3FFF;
            self.mem_write(DIV, (self.system_counter >> 6) as u8);
        }

        let new_signal = self.timer_signal(self.system_counter, self.mem_read(TAC));
        if prev_signal && !new_signal {
            self.increment_tima();
        }

        self.prev_signal = new_signal;
    }

    fn increment_tima(&mut self) {
        let tima = self.mem_read(TIMA);
        let (result, overflow) = tima.overflowing_add(1);

        if overflow {
            self.mem_write(TIMA, 0);
            self.overflow_delay = 4;
        } else {
            self.mem_write(TIMA, result);
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

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.with_mem(|mem| mem.dbg_read(addr))
    }
    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.with_mem_mut(|mem| mem.dbg_write(addr, data));
    }

    pub fn check_write_div(&mut self) -> bool {
        self.with_mem_mut(|mem| mem.check_write_div())
    }

    pub fn check_write_tac(&mut self) -> bool {
        self.with_mem_mut(|mem| mem.check_write_tac())
    }
}
