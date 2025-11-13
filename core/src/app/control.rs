use crate::emu::joypad::JoypadButton;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMessage {
    Exit,
    JoypadInput {
        button: JoypadButton,
        pressed: bool,
    },
}

pub type ControlSender = Sender<ControlMessage>;
pub type ControlReceiver = Receiver<ControlMessage>;
