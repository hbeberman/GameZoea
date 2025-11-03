use crate::emu::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Timer {
    mem: Rc<RefCell<Memory>>,
}

impl Timer {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        Timer { mem }
    }

    pub fn tick(&mut self, t: u128) {
        if t.is_multiple_of(256) {
            let div = self.mem_dbg_read(0xFF04);
            let (result, _ ) = div.overflowing_add(1);
            // TODO: Increment TIMA and fire interrupts in response
            self.mem_dbg_write(0xFF04, result);
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

}
