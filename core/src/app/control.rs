use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMessage {
    Exit,
}

pub type ControlSender = Sender<ControlMessage>;
pub type ControlReceiver = Receiver<ControlMessage>;
