use crate::emu::gb::Comp;
use crate::emu::mem::Memory;
use crate::emu::regs::*;
use crate::{bit, setbit};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoypadButton {
    Right,
    Left,
    Up,
    Down,
    A,
    B,
    Start,
    Select,
}

#[derive(Debug, Clone, Copy)]
struct JoypadEvent {
    button: JoypadButton,
    pressed: bool,
}

#[derive(Default)]
struct JoypadState {
    right: bool,
    left: bool,
    up: bool,
    down: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
}

impl JoypadState {
    fn set(&mut self, button: JoypadButton, pressed: bool) -> bool {
        let state = match button {
            JoypadButton::Right => &mut self.right,
            JoypadButton::Left => &mut self.left,
            JoypadButton::Up => &mut self.up,
            JoypadButton::Down => &mut self.down,
            JoypadButton::A => &mut self.a,
            JoypadButton::B => &mut self.b,
            JoypadButton::Start => &mut self.start,
            JoypadButton::Select => &mut self.select,
        };

        let was_pressed = *state;
        *state = pressed;
        !was_pressed && pressed
    }

    fn directions_column(&self) -> u8 {
        let mut value = 0x0F;
        if self.right {
            value &= !bit!(0);
        }
        if self.left {
            value &= !bit!(1);
        }
        if self.up {
            value &= !bit!(2);
        }
        if self.down {
            value &= !bit!(3);
        }
        value
    }

    fn buttons_column(&self) -> u8 {
        let mut value = 0x0F;
        if self.a {
            value &= !bit!(0);
        }
        if self.b {
            value &= !bit!(1);
        }
        if self.select {
            value &= !bit!(2);
        }
        if self.start {
            value &= !bit!(3);
        }
        value
    }
}

pub struct Joypad {
    mem: Rc<RefCell<Memory>>,
    queue: VecDeque<JoypadEvent>,
    state: JoypadState,
}

impl Joypad {
    pub fn init_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        let mut joypad = Joypad {
            mem,
            queue: VecDeque::new(),
            state: JoypadState::default(),
        };

        joypad.mem_write(P1, 0xCF);

        joypad
    }

    pub fn tick(&mut self, t: u128) {
        self.own(true);

        if t.is_multiple_of(4) {
            self.check_queue();
        }

        self.own(false);
    }

    pub fn enqueue_input(&mut self, button: JoypadButton, pressed: bool) {
        self.queue.push_back(JoypadEvent { button, pressed });
    }

    fn check_queue(&mut self) {
        let mut request_interrupt = false;
        while let Some(event) = self.queue.pop_front() {
            if self.state.set(event.button, event.pressed) {
                request_interrupt |= event.pressed;
            }
        }

        self.update_p1();

        if request_interrupt {
            let mut reg_if = self.mem_read(IF);
            setbit!(reg_if, 4);
            self.mem_write(IF, reg_if);
        }
    }

    fn update_p1(&mut self) {
        let cur = self.mem_read(P1);
        let mut column = 0x0F;

        if cur & bit!(4) == 0 {
            column &= self.state.directions_column();
        }

        if cur & bit!(5) == 0 {
            column &= self.state.buttons_column();
        }

        let upper = (cur & 0x30) | 0xC0;
        self.mem_write(P1, upper | column);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emu::mem::Memory;

    #[test]
    fn joypad_updates_p1_when_button_pressed() {
        let mem = Rc::new(RefCell::new(Memory::empty()));
        let mut joypad = Joypad::init_dmg(mem.clone());

        // CPU selects the directions column (bit 4 low)
        {
            let mut mem_mut = mem.borrow_mut();
            mem_mut.set_addr(P1);
            mem_mut.set_data(0x20);
            mem_mut.write();
        }

        joypad.enqueue_input(JoypadButton::Right, true);
        joypad.check_queue();

        let value = mem.borrow().dbg_read(P1);
        assert_eq!(value & 0x0F, 0x0E, "Right press should clear bit 0");
    }
}
