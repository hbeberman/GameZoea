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
}

impl Timer {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Timer {
            mem,
            system_counter: 0,
            internal_tma: 0,
        }
    }

    pub fn tick(&mut self, t: u128) {
        if self.check_write_div() {
            self.system_counter = 0;
        }

        let snapshot = self.system_counter;
        if t.is_multiple_of(4) {
            let (result, _) = self.system_counter.overflowing_add(1);
            self.system_counter = result;
            self.mem_dbg_write(DIV, (result >> 8) as u8);
        }

        let mut tima = self.mem_dbg_read(TIMA);
        let tma = self.mem_dbg_read(TMA);
        let tac = self.mem_dbg_read(TAC);

        let delta = snapshot ^ self.system_counter;

        let falling_match = match tac & 0x3 {
            0x0 => 0x80,
            0x1 => 0x02,
            0x2 => 0x08,
            0x3 => 0x20,
            _ => unreachable!(),
        };

        if falling_match & delta != 0x0 && tac & 0x4 == 0x4 {
            let (result, overflow) = tima.overflowing_add(1);

            if overflow {
                self.mem_dbg_write(IF, self.mem_dbg_read(IF) | 0x4);
                tima = self.internal_tma;
            } else {
                tima = result;
            }
        }

        self.mem_dbg_write(TIMA, tima);

        if t.is_multiple_of(4) {
            self.internal_tma = tma;
        }
        self.mem_dbg_write(TAC, tac);
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
}
