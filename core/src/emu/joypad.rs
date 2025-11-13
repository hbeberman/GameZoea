use crate::emu::gb::Comp;
use crate::emu::mem::Memory;
use crate::{bit, clearbit, setbit};
use std::cell::RefCell;
use std::rc::Rc;

const IFLAG: u16 = 0xFF0F;

pub const P1: u16 = 0xFF01;

pub struct Joypad {
    mem: Rc<RefCell<Memory>>,
}

impl Joypad {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        let mut joypad = Joypad { mem };

        joypad.mem_write(0xFF00, 0x3F);

        joypad
    }

    pub fn tick(&mut self, t: u128) {
        self.own(true);

        if t.is_multiple_of(4) {
            self.check_queue();
        }

        self.own(false);
    }

    fn check_queue(&mut self) {
        let cur = self.mem_read(0xFF00);

        let mut reg_if = self.mem_read(IFLAG);
        setbit!(reg_if, 2);
        self.mem_write(IFLAG, reg_if);
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
        let owner = if own { Comp::Joypad } else { Comp::None };
        self.with_mem_mut(|mem| mem.set_owner(owner))
    }
}
