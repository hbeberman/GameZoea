#![allow(dead_code)]
use crate::emu::cpu::*;
use macros::*;

mod emu;

fn main() {
    let cpu = Cpu::default();
    println!("GameZoea!\n{}", cpu);
    const ROM: &[u8] = gbasm! {r#"
        inc a
    "#};
    println!("ROM: {:02x?}", ROM);
}
