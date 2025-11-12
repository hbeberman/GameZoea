use gamezoea::emu::gb::*;
use macros::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gamezoea::*;

    #[test]
    fn blargg_cpu_instrs_individual_01_special() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/01-special.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(1000000);
        eprintln!("blargg {}", gb.serial.buffmt());
        eprintln!("blargg {:?}", gb.serial.buf);
        assert_hex_eq!(gb.cpu.a(), 0x01);
    }
}
