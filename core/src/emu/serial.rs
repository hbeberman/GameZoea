use crate::emu::gb::Comp;
use crate::emu::mem::Memory;
use crate::{clearbit, isbitset};
use std::cell::RefCell;
use std::rc::Rc;

pub const SB: u16 = 0xFF01;
pub const SC: u16 = 0xFF02;

pub struct Serial {
    mem: Rc<RefCell<Memory>>,
    pub buf: Vec<u8>,
}

impl Serial {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        let sb = 0x00;
        let sc = 0x00;
        let mut serial = Serial {
            mem,
            buf: Vec::new(),
        };

        serial.mem_write(SB, sb);
        serial.mem_write(SC, sc);
        serial
    }

    pub fn tick(&mut self, t: u128) {
        self.own(true);

        if t.is_multiple_of(4) {
            self.transfer();
        }

        self.own(false);
    }

    fn transfer(&mut self) {
        let mut sc = self.mem_read(SC);
        let sb = self.mem_read(SB);

        // TODO: actually do timed serial transfers
        if isbitset!(sc, 7) {
            self.buf.push(sb);
        }

        clearbit!(sc, 7);
        self.mem_write(SC, sc);
    }

    pub fn buffmt(&self) -> String {
        String::from_utf8_lossy(&self.buf).to_string()
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

    pub fn own(&mut self, own: bool) {
        let owner = if own { Comp::Serial } else { Comp::None };
        self.with_mem_mut(|mem| mem.set_owner(owner))
    }
}
