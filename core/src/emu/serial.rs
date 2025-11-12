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

        if isbitset!(sc, 7) {
            eprintln!("serial: {:02X}", sb);
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

    // TODO: delete
    pub fn set_tima_overflow(&mut self, tima_overflow: bool) {
        self.with_mem_mut(|mem| mem.set_tima_overflow(tima_overflow))
    }

    pub fn own(&mut self, own: bool) {
        let owner = if own { Comp::Serial } else { Comp::None };
        self.with_mem_mut(|mem| mem.set_owner(owner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_blargg;

    #[test]
    fn test_buffmt_ascii() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut serial = Serial::init_dmg(mem);

        // Add some ASCII bytes to the buffer
        serial.buf = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"

        assert_eq!(serial.buffmt(), "Hello");
    }

    #[test]
    fn test_buffmt_empty() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let serial = Serial::init_dmg(mem);

        assert_eq!(serial.buffmt(), "");
    }

    #[test]
    fn test_buffmt_with_newline() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut serial = Serial::init_dmg(mem);

        // "Test\n"
        serial.buf = vec![0x54, 0x65, 0x73, 0x74, 0x0A];

        assert_eq!(serial.buffmt(), "Test\n");
    }

    #[test]
    fn test_buffmt_non_ascii() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut serial = Serial::init_dmg(mem);

        // Mix of ASCII and non-UTF8 bytes
        serial.buf = vec![0x41, 0x42, 0xFF, 0x43]; // "AB�C" (FF is invalid UTF-8)

        let result = serial.buffmt();
        assert!(result.starts_with("AB"));
        assert!(result.ends_with("C"));
    }

    #[test]
    fn test_assert_blargg_success() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut serial = Serial::init_dmg(mem);

        serial.buf = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"

        // This should pass
        assert_blargg!(serial.buffmt(), "Hello");
    }

    #[test]
    #[should_panic(expected = "Serial buffer mismatch")]
    fn test_assert_blargg_failure() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut serial = Serial::init_dmg(mem);

        serial.buf = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello"

        // This should fail
        assert_blargg!(serial.buffmt(), "Goodbye");
    }
}
