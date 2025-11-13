use gamezoea::emu::gb::*;
use macros::*;

#[cfg(test)]
mod tests {
    use super::*;
    use gamezoea::*;

    #[test]
    fn blargg_cpu_instrs_individual_01_special() {
        let result = "01-special";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/01-special.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_02_interrupts() {
        let result = "02-interrupts";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/02-interrupts.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_03_op_sp_hl() {
        let result = "03-op sp,hl";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/03-op sp,hl.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_04_op_r_imm() {
        let result = "04-op r,imm";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/04-op r,imm.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_05_op_rp() {
        let result = "05-op rp";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/05-op rp.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_06_ld_r_r() {
        let result = "06-ld r,r";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/06-ld r,r.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_07_jr_jp_call_ret_rst() {
        let result = "07-jr,jp,call,ret,rst";
        const ROM: &[u8] =
            gbrom!("tests/roms/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_08_misc_instrs_gb() {
        let result = "08-misc instrs";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/08-misc instrs.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_09_op_r_r() {
        let result = "09-op r,r";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/09-op r,r.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_10_bit_ops() {
        let result = "10-bit ops";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/10-bit ops.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_cpu_instrs_individual_11_op_a_mhl() {
        let result = "11-op a,(hl)";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/cpu_instrs/individual/11-op a,(hl).gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_instr_timing() {
        let result = "instr_timing";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/instr_timing/instr_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
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
        let result = "01-read_timing";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/01-read_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_mem_timing_individual_02_write_timing() {
        let result = "02-write_timing";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/02-write_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    fn blargg_mem_timing_individual_03_modify_timing() {
        let result = "03-modify_timing";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/mem_timing/individual/03-modify_timing.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }

    #[test]
    #[ignore = "TODO: implement ei and rst halt bugs"]
    fn blargg_halt_bug() {
        let result = "halt_bug";
        const ROM: &[u8] = gbrom!("tests/roms/blargg/halt_bug/halt_bug.gb");
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(8000000);
        gb.step_blargg(8000000, result);
        assert_blargg!(gb.serial.buffmt(), result);
    }
}
