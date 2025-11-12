use gamezoea::emu::gb::*;
use macros::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gamezoea::*;

    #[test]
    #[ignore = "TODO"]
    fn mooneye_acceptance_0() {
        const ROM: &[u8] = gbasm! {r#"
  halt
    "#};
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(20000);
        assert_hex_eq!(gb.cpu.a(), 0x01);
    }

    #[test]
    #[allow(non_snake_case)]
    #[ignore = "TODO"]
    fn mooneye_boot_hwio_dmgABCmgb() {
        const ROM: &[u8] = gbrom!("tests/roms/mooneye/acceptance/boot_hwio-dmgABCmgb.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(100000);
        eprintln!("mooneye {}", gb.serial.buffmt());
        panic!();
    }
}
