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
        gb.step(2000000);
        assert_blargg!(gb.serial.buffmt(), "01-special");
    }

    #[test]
    fn blargg_cpu_instrs_individual_02_interrupts() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/02-interrupts.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(200000);
        assert_blargg!(gb.serial.buffmt(), "02-interrupts");
    }

    #[test]
    fn blargg_cpu_instrs_individual_03_op_sp_hl() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/03-op sp,hl.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(2000000);
        assert_blargg!(gb.serial.buffmt(), "03-op sp,hl");
    }

    #[test]
    fn blargg_cpu_instrs_individual_04_op_r_imm() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/04-op r,imm.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(2000000);
        assert_blargg!(gb.serial.buffmt(), "04-op r,imm");
    }

    #[test]
    fn blargg_cpu_instrs_individual_05_op_rp() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/05-op rp.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(2000000);
        assert_blargg!(gb.serial.buffmt(), "05-op rp");
    }

    #[test]
    fn blargg_cpu_instrs_individual_06_ld_r_r() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/06-ld r,r.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(1000000);
        assert_blargg!(gb.serial.buffmt(), "06-ld r,r");
    }

    #[test]
    fn blargg_cpu_instrs_individual_07_jr_jp_call_ret_rst() {
        const ROM: &[u8] =
            gbrom!("tests/roms/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(1000000);
        assert_blargg!(gb.serial.buffmt(), "07-jr,jp,call,ret,rst");
    }

    #[test]
    fn blargg_cpu_instrs_individual_08_misc_instrs_gb() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/08-misc instrs.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(1000000);
        assert_blargg!(gb.serial.buffmt(), "08-misc instrs");
    }

    #[test]
    fn blargg_cpu_instrs_individual_09_op_r_r() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/09-op r,r.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(5000000);
        assert_blargg!(gb.serial.buffmt(), "09-op r,r");
    }

    #[test]
    fn blargg_cpu_instrs_individual_10_bit_ops() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/10-bit ops.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(8000000);
        assert_blargg!(gb.serial.buffmt(), "10-bit ops");
    }

    #[test]
    fn blargg_cpu_instrs_individual_11_op_a_mhl() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/11-op a,(hl).gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(8000000);
        assert_blargg!(gb.serial.buffmt(), "11-op a,(hl)");
    }

    #[test]
    fn blargg_instr_timing() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/instr_timing/instr_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(800000);
        assert_blargg!(gb.serial.buffmt(), "instr_timing");
    }

    #[test]
    #[ignore = "TODO"]
    fn blargg_interrupt_time() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/interrupt_time/interrupt_time.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(800000);
        assert_blargg!(gb.serial.buffmt(), "interrupt_time");
    }

    #[test]
    fn blargg_mem_timing_individual_01_read_timing() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/01-read_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(800000);
        assert_blargg!(gb.serial.buffmt(), "01-read_timing");
    }

    #[test]
    fn blargg_mem_timing_individual_02_write_timing() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/02-write_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(800000);
        assert_blargg!(gb.serial.buffmt(), "02-write_timing");
    }

    #[test]
    fn blargg_mem_timing_individual_03_modify_timing() {
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/03-modify_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(800000);
        assert_blargg!(gb.serial.buffmt(), "03-modify_timing");
    }
}
