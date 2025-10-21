use gamezoea::emu::cpu::*;
use macros::*;
// {{{ Tests
#[cfg(test)]
mod tests {
    use super::*;

    // {{{ Register Tests
    #[test]
    fn cpu_default() {
        let cpu = Cpu::default();

        assert_eq!(cpu.m(), 0);
        assert_eq!(cpu.t(), 0);
        assert_eq!(cpu.af(), 0x0000);
        assert_eq!(cpu.bc(), 0x0000);
        assert_eq!(cpu.de(), 0x0000);
        assert_eq!(cpu.hl(), 0x0000);
        assert_eq!(cpu.sp(), 0x0000);
        assert_eq!(cpu.pc(), 0x0000);
    }

    #[test]
    fn cpu_setters() {
        let mut cpu = Cpu::default();
        cpu.set_m(1337);
        cpu.set_t(5348);
        cpu.set_addr(0xCCCC);
        cpu.set_data(0xDD);
        cpu.set_ir(0xAA);
        cpu.set_ie(0xBB);
        cpu.set_af(0x1020);
        cpu.set_bc(0x3040);
        cpu.set_de(0x5060);
        cpu.set_hl(0x7080);
        cpu.set_sp(0x90A0);
        cpu.set_pc(0xB0C0);

        assert_eq!(cpu.m(), 1337);
        assert_eq!(cpu.t(), 5348);
        assert_eq!(cpu.addr(), 0xCCCC);
        assert_eq!(cpu.data(), 0xDD);
        assert_eq!(cpu.ir(), 0xAA);
        assert_eq!(cpu.ie(), 0xBB);
        assert_eq!(cpu.af(), 0x1020);
        assert_eq!(cpu.bc(), 0x3040);
        assert_eq!(cpu.de(), 0x5060);
        assert_eq!(cpu.hl(), 0x7080);
        assert_eq!(cpu.sp(), 0x90A0);
        assert_eq!(cpu.pc(), 0xB0C0);
    }

    #[test]
    fn cpu_flag_sets() {
        let mut cpu = Cpu::default();
        cpu.set_zero(1);
        cpu.set_bcdn(1);
        cpu.set_bcdh(1);
        cpu.set_carry(1);
        assert_eq!(cpu.f(), 0xF0);
    }

    #[test]
    fn cpu_flag_gets() {
        let mut cpu = Cpu::default();
        cpu.set_af(0x00F0);
        assert_eq!(cpu.zero(), 1);
        assert_eq!(cpu.bcdn(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.carry(), 1);
    }

    #[test]
    #[should_panic(expected = "Invalid value used as flag z: 02")]
    fn cpu_flag_z_invalid() {
        let mut cpu = Cpu::default();
        cpu.set_zero(2);
        assert_eq!(cpu.zero(), 1);
    }
    // }}}

    // {{{ Cycle Tests
    #[test]
    fn cpu_t_tick() {
        let mut cpu = Cpu::default();
        assert_eq!(cpu.t(), 0);
        for i in 1..16 {
            cpu.tick_t1();
            assert_eq!(cpu.t(), i);
            assert_eq!(cpu.m(), i / 4);
        }
    }
    // }}}

    // {{{ Memory Tests
    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to ROM bank 00: 0000:ab")]
    fn mem_rom_write() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0x0000);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to ROM bank 01-NN: 4000:ab")]
    fn mem_rom_bankable_write() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0x4000);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    fn mem_write_read_vram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0x8000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_eq!(cpu.mem_dbg_read(0x8000), 0xAB);
    }

    #[test]
    fn mem_write_read_external_ram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xA000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_eq!(cpu.mem_dbg_read(0xA000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xC000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_eq!(cpu.mem_dbg_read(0xC000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram_bankable() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xD000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_eq!(cpu.mem_dbg_read(0xD000), 0xAB);
    }

    #[test]
    #[should_panic(expected = "Memory write to echo RAM: e000:ab")]
    fn mem_write_echo() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xE000);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to OAM: fe00:ab")]
    fn mem_write_oam() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFE00);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "Memory write to not usable: fea0:ab")]
    fn mem_write_not_usable() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFEA0);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to I/O registers: ff00:ab")]
    fn mem_write_io() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFF00);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to HRAM: ff80:ab")]
    fn mem_write_hram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFF80);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to IE register: ffff:ab")]
    fn mem_write_ie() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFFFF);
        cpu.set_data(0xAB);
        cpu.mem_write();
    }
    // }}}

    // {{{ Execute Tests
    // {{{ test execute_nop
    #[test]
    fn execute_nop() {
        const ROM: &[u8] = gbasm! {r#"
    nop
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0152);
    }
    // }}}

    // {{{ test execute_ld_r16_imm16
    #[test]
    fn execute_ld_r16_imm16() {
        const ROM: &[u8] = gbasm! {r#"
  ld bc, 0x0102
  ld de, 0x0304
  ld hl, 0x0506
  ld sp, 0x0708
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x015D);
        assert_eq!(cpu.bc(), 0x0102);
        assert_eq!(cpu.de(), 0x0304);
        assert_eq!(cpu.hl(), 0x0506);
        assert_eq!(cpu.sp(), 0x0708);
    }
    // }}}

    // {{{ test execute_ld_mr16mem_a
    #[test]
    fn execute_ld_mr16mem_a() {
        const ROM: &[u8] = gbasm! {r#"
  ld bc, 0xC000
  ld de, 0xD000
  ld hl, 0xD0F0
  inc a
  ld [bc], a
  ld [de], a
  ld [hl+], a
  ld [hl-], a
  ld [hl-], a
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.mem_dbg_read(0xC000), 0x2);
        assert_eq!(cpu.mem_dbg_read(0xD000), 0x2);
        assert_eq!(cpu.hl(), 0xD0EF);
        assert_eq!(cpu.mem_dbg_read(0xD0EF), 0x0);
        assert_eq!(cpu.mem_dbg_read(0xD0F0), 0x2);
        assert_eq!(cpu.mem_dbg_read(0xD0F1), 0x2);
        assert_eq!(cpu.mem_dbg_read(0xD0F2), 0x0);
    }
    // }}}

    // {{{ test execute_inc_r16
    #[test]
    fn execute_inc_r16() {
        const ROM: &[u8] = gbasm! {r#"
  inc bc
  inc de
  inc hl
  inc sp
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0155);
        assert_eq!(cpu.sp(), 0xFFFF);
        assert_eq!(cpu.a(), 0x01);
        assert_eq!(cpu.bc(), 0x0014);
        assert_eq!(cpu.de(), 0x00D9);
        assert_eq!(cpu.hl(), 0x014E);
        assert_eq!(cpu.zero(), 1);
        assert_eq!(cpu.carry(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test execute_dec_r16
    #[test]
    fn execute_dec_r16() {
        const ROM: &[u8] = gbasm! {r#"
  dec bc
  dec de
  dec hl
  dec sp
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0155);
        assert_eq!(cpu.sp(), 0xFFFD);
        assert_eq!(cpu.a(), 0x01);
        assert_eq!(cpu.bc(), 0x0012);
        assert_eq!(cpu.de(), 0x00D7);
        assert_eq!(cpu.hl(), 0x014C);
        assert_eq!(cpu.zero(), 1);
        assert_eq!(cpu.carry(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test execute_add_hl_r16
    #[test]
    fn execute_add_hl_r16() {
        const ROM: &[u8] = gbasm! {r#"
  add hl, bc
  add hl, de
  add hl, hl
  add hl, sp
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0155);
        assert_eq!(cpu.sp(), 0xFFFE);
        assert_eq!(cpu.a(), 0x01);
        assert_eq!(cpu.bc(), 0x0013);
        assert_eq!(cpu.de(), 0x00D8);
        assert_eq!(cpu.hl(), 0x046E);
        assert_eq!(cpu.zero(), 1);
        assert_eq!(cpu.carry(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test execute_inc_r8
    #[test]
    fn execute_inc_r8() {
        const ROM: &[u8] = gbasm! {r#"
  inc a
  inc l
  inc h
  inc e
  inc d
  inc c
  inc b
  inc l
  inc l
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x015A);
        assert_eq!(cpu.sp(), 0xFFFE);
        assert_eq!(cpu.a(), 0x02);
        assert_eq!(cpu.bc(), 0x0114);
        assert_eq!(cpu.de(), 0x01D9);
        assert_eq!(cpu.hl(), 0x0250);
        assert_eq!(cpu.zero(), 0);
        assert_eq!(cpu.carry(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test execute_dec_r8
    #[test]
    fn execute_dec_r8() {
        const ROM: &[u8] = gbasm! {r#"
  dec a
  dec l
  dec h
  dec e
  dec d
  dec c
  dec b
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0158);
        assert_eq!(cpu.sp(), 0xFFFE);
        assert_eq!(cpu.a(), 0x00);
        assert_eq!(cpu.bc(), 0xFF12);
        assert_eq!(cpu.de(), 0xFFD7);
        assert_eq!(cpu.hl(), 0x004C);
        assert_eq!(cpu.zero(), 0);
        assert_eq!(cpu.carry(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.bcdn(), 1);
    }
    // }}}

    // {{{ test halt
    #[test]
    fn execute_halt() {
        const ROM: &[u8] = gbasm! {r#"
  inc a
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_eq!(cpu.pc(), 0x0152);
    }
    // }}}

    // {{{ test jp_imm16
    #[test]
    fn execute_jp_imm16() {
        const ROM: &[u8] = gbasm! {r#"
  jp SkipIncA
  inc a
Backwards:
  inc c
  halt
SkipIncA:
  inc b
  jp Backwards
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(30);
        assert_eq!(cpu.bc(), 0x0114);
        assert_eq!(cpu.pc(), 0x0156);
        assert_eq!(cpu.a(), 0x01);
    }
    // }}}
}
// }}}
