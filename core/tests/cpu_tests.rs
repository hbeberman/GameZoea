use gamezoea::emu::cpu::*;
use macros::*;
macro_rules! assert_hex_eq {
    ($a:expr, $b:expr) => {
        assert!($a == $b, "assertion failed: {:#06x} != {:#06x}", $a, $b);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // {{{ Register Tests
    #[test]
    fn cpu_default() {
        let cpu = Cpu::default();

        assert_hex_eq!(cpu.m(), 0);
        assert_hex_eq!(cpu.t(), 0);
        assert_hex_eq!(cpu.af(), 0x0000);
        assert_hex_eq!(cpu.bc(), 0x0000);
        assert_hex_eq!(cpu.de(), 0x0000);
        assert_hex_eq!(cpu.hl(), 0x0000);
        assert_hex_eq!(cpu.sp(), 0x0000);
        assert_hex_eq!(cpu.pc(), 0x0000);
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

        assert_hex_eq!(cpu.m(), 1337);
        assert_hex_eq!(cpu.t(), 5348);
        assert_hex_eq!(cpu.addr(), 0xCCCC);
        assert_hex_eq!(cpu.data(), 0xDD);
        assert_hex_eq!(cpu.ir(), 0xAA);
        assert_hex_eq!(cpu.ie(), 0xBB);
        assert_hex_eq!(cpu.af(), 0x1020);
        assert_hex_eq!(cpu.bc(), 0x3040);
        assert_hex_eq!(cpu.de(), 0x5060);
        assert_hex_eq!(cpu.hl(), 0x7080);
        assert_hex_eq!(cpu.sp(), 0x90A0);
        assert_hex_eq!(cpu.pc(), 0xB0C0);
    }

    #[test]
    fn cpu_flag_sets() {
        let mut cpu = Cpu::default();
        cpu.set_zero(1);
        cpu.set_bcdn(1);
        cpu.set_bcdh(1);
        cpu.set_carry(1);
        assert_hex_eq!(cpu.f(), 0xF0);
    }

    #[test]
    fn cpu_flag_gets() {
        let mut cpu = Cpu::default();
        cpu.set_af(0x00F0);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }

    #[test]
    #[should_panic(expected = "Invalid value used as flag z: 02")]
    fn cpu_flag_z_invalid() {
        let mut cpu = Cpu::default();
        cpu.set_zero(2);
        assert_hex_eq!(cpu.zero(), 1);
    }
    // }}}

    // {{{ Cycle Tests
    #[test]
    fn cpu_t_tick() {
        let mut cpu = Cpu::default();
        assert_hex_eq!(cpu.t(), 0);
        for i in 1..16 {
            cpu.tick_t1();
            assert_hex_eq!(cpu.t(), i);
            assert_hex_eq!(cpu.m(), i / 4);
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
        assert_hex_eq!(cpu.mem_dbg_read(0x8000), 0xAB);
    }

    #[test]
    fn mem_write_read_external_ram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xA000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_hex_eq!(cpu.mem_dbg_read(0xA000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xC000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram_bankable() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xD000);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_hex_eq!(cpu.mem_dbg_read(0xD000), 0xAB);
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
    fn mem_write_hram() {
        let mut cpu = Cpu::default();
        cpu.set_addr(0xFF80);
        cpu.set_data(0xAB);
        cpu.mem_write();
        assert_hex_eq!(cpu.mem_dbg_read(0xFF80), 0xAB);
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

    // {{{ test execute_nop
    #[test]
    fn execute_nop() {
        const ROM: &[u8] = gbasm! {r#"
  nop
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.pc(), 0x0152);
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
        assert_hex_eq!(cpu.pc(), 0x015D);
        assert_hex_eq!(cpu.bc(), 0x0102);
        assert_hex_eq!(cpu.de(), 0x0304);
        assert_hex_eq!(cpu.hl(), 0x0506);
        assert_hex_eq!(cpu.sp(), 0x0708);
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
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0x2);
        assert_hex_eq!(cpu.mem_dbg_read(0xD000), 0x2);
        assert_hex_eq!(cpu.hl(), 0xD0EF);
        assert_hex_eq!(cpu.mem_dbg_read(0xD0EF), 0x0);
        assert_hex_eq!(cpu.mem_dbg_read(0xD0F0), 0x2);
        assert_hex_eq!(cpu.mem_dbg_read(0xD0F1), 0x2);
        assert_hex_eq!(cpu.mem_dbg_read(0xD0F2), 0x0);
    }
    // }}}

    // {{{ test execute_ld_a_mr16mem
    #[test]
    fn execute_ld_a_mr16mem() {
        const ROM: &[u8] = gbasm! {r#"
  inc a
  ld bc, 0xC000
  ld de, 0xC001
  ld hl, 0xC005
  ld [bc], a
  dec a
  ld a, [bc]
  ld [de], a 
  dec a
  ld a, [de]
  ld [hl-], a
  dec a
  ld hl, 0xC005
  ld a, [hl+]
  dec a
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x01);
    }
    // }}}

    // {{{ test execute_ld_mimm16_sp
    #[test]
    fn execute_ld_mimm16_sp() {
        const ROM: &[u8] = gbasm! {r#"
  ld [0xC000], sp
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.pc(), 0x0154);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0xFE);
        assert_hex_eq!(cpu.mem_dbg_read(0xC001), 0xFF);
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
        assert_hex_eq!(cpu.pc(), 0x0155);
        assert_hex_eq!(cpu.sp(), 0xFFFF);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.bc(), 0x0014);
        assert_hex_eq!(cpu.de(), 0x00D9);
        assert_hex_eq!(cpu.hl(), 0x014E);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.carry(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 0);
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
        assert_hex_eq!(cpu.pc(), 0x0155);
        assert_hex_eq!(cpu.sp(), 0xFFFD);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.bc(), 0x0012);
        assert_hex_eq!(cpu.de(), 0x00D7);
        assert_hex_eq!(cpu.hl(), 0x014C);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.carry(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 0);
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
        assert_hex_eq!(cpu.pc(), 0x0155);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.bc(), 0x0013);
        assert_hex_eq!(cpu.de(), 0x00D8);
        assert_hex_eq!(cpu.hl(), 0x046E);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.carry(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test execute_inc_r8_and_inc_mhl
    #[test]
    fn execute_inc_r8_and_inc_mhl() {
        const ROM: &[u8] = gbasm! {r#"
  inc a
  inc l
  inc h
  inc e
  inc d
  inc c
  inc b
  ld hl, 0xC000
  ld [hl], 0x00
  inc [hl]
  ld hl, 0x024e
  inc l
  inc l
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.bc(), 0x0114);
        assert_hex_eq!(cpu.de(), 0x01D9);
        assert_hex_eq!(cpu.hl(), 0x0250);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.carry(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0x01);
    }
    // }}}

    // {{{ test execute_dec_r8_and_dec_mhl
    #[test]
    fn execute_dec_r8_and_dec_mhl() {
        const ROM: &[u8] = gbasm! {r#"
  dec a
  dec e
  dec d
  dec c
  ld hl, 0xC000
  ld [hl], 0x00
  dec [hl]
  dec l
  dec h
  dec b
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.a(), 0x00);
        assert_hex_eq!(cpu.bc(), 0xFF12);
        assert_hex_eq!(cpu.de(), 0xFFD7);
        assert_hex_eq!(cpu.hl(), 0xBFFF);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.carry(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0xFF);
    }
    // }}}

    // {{{ test ld_r8_imm8_and_ld_mhl_imm8
    #[test]
    fn execute_ld_r8_imm8_and_ld_mhl_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xc000
  ld [hl], 0xA5
  ld b, 0x01
  ld c, 0x02
  ld d, 0x03
  ld e, 0x04
  ld h, 0x05
  ld l, 0x06
  ld a, 0x00
            "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.pc(), 0x0164);
        assert_hex_eq!(cpu.bc(), 0x0102);
        assert_hex_eq!(cpu.de(), 0x0304);
        assert_hex_eq!(cpu.hl(), 0x0506);
        assert_hex_eq!(cpu.a(), 0x00);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0xA5);
    }
    // }}}

    // {{{ test rlca
    #[test]
    fn execute_rlca() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  rlca
    "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x4B);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test rrca
    #[test]
    fn execute_rrca() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0x22
  rrca
  rrca
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x88);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test rla
    #[test]
    fn execute_rla() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0x44
  rla
  rla
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x12);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test rra
    #[test]
    fn execute_rra() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0x22
  rra
  rra
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x48);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test daa
    #[test]
    #[ignore = "TODO"]
    fn execute_daa() {
        const ROM: &[u8] = gbasm! {r#"
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x00);
    }
    // }}}

    // {{{ test cpl
    #[test]
    fn execute_cpl() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  cpl
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x5A);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.bcdn(), 1);
    }
    // }}}

    // {{{ test scf
    #[test]
    fn execute_scf() {
        const ROM: &[u8] = gbasm! {r#"
  inc a
  scf
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test ccf
    #[test]
    fn execute_ccf() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0x10
  dec a
  ccf
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x0F);
        assert_hex_eq!(cpu.carry(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 0);
    }
    // }}}

    // {{{ test jr_imm8
    #[test]
    fn execute_jr_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  jr .skipinc
  inc a
.deadend
  inc de
  inc bc
  halt
.skipinc
  inc hl
  jr .deadend
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.pc(), 0x0156);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.bc(), 0x0014);
        assert_hex_eq!(cpu.de(), 0x00D9);
        assert_hex_eq!(cpu.hl(), 0x014E);
    }
    // }}}

    // {{{ test jr_cond_imm8
    #[test]
    fn execute_jr_cond_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  dec a
  jr z,.ztaken
  inc a
.backjump:
  inc d
  halt
.ztaken:
  dec a
  jr nz,.nztaken
  inc c
.nztaken:
  ld a,$FE
  inc a
  jr nc,.nctaken
  inc e
.nctaken:
  inc h
  jr c,.ctaken
  inc l
.ctaken:
  jr nz,.backjump
  halt
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.pc(), 0x0156);
        assert_hex_eq!(cpu.a(), 0xFF);
        assert_hex_eq!(cpu.bc(), 0x0013);
        // This is actually a problem, something is clearing carry
        assert_hex_eq!(cpu.de(), 0x01D9);
        assert_hex_eq!(cpu.hl(), 0x024D);
    }
    // }}}

    // {{{ test stop
    #[test]
    #[ignore = "TODO"]
    fn execute_stop() {
        const ROM: &[u8] = gbasm! {r#"
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x00);
    }
    // }}}

    // {{{ test ld_r8_r8
    #[test]
    fn execute_ld_r8_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld b, a
  ld c, b
  ld d, c
  ld e, d
  ld h, e
  ld l, h
  inc a
  ld a, l
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.b(), 0x01);
        assert_hex_eq!(cpu.c(), 0x01);
        assert_hex_eq!(cpu.d(), 0x01);
        assert_hex_eq!(cpu.e(), 0x01);
        assert_hex_eq!(cpu.h(), 0x01);
        assert_hex_eq!(cpu.l(), 0x01);
    }
    // }}}

    // {{{ test ld_r8_mhl_and_ld_mhl_r8
    #[test]
    fn execute_ld_r8_mhl_and_ld_mhl_r8() {
        const ROM: &[u8] = gbasm! {r#"
    ld hl, 0xC000
    ld [hl], a
    ld b, [hl]
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0x01);
        assert_hex_eq!(cpu.b(), 0x01);
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
        assert_hex_eq!(cpu.pc(), 0x0152);
    }
    // }}}

    // {{{ test add_a_r8
    #[test]
    fn execute_add_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld [hl], c
  add a, d
  add a, [hl]
  add a, a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x28);
        assert_hex_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test adc_a_r8
    #[test]
    fn execute_adc_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0xff
  ld [hl], b
  adc a, [hl]
  adc a, b
  adc a, e
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0xDA);
        assert_hex_eq!(cpu.bcdn(), 0);
    }
    // }}}

    // {{{ test sub_a_r8
    #[test]
    fn execute_sub_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0xff
  ld [hl], b
  sub a, [hl]
  sub a, b
  sub a, e
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x2B);
        assert_hex_eq!(cpu.bcdn(), 1);
    }
    // }}}

    // {{{ test sbc_a_r8
    #[test]
    fn execute_sbc_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0xff
  ld [hl], b
  sbc a, [hl]
  sbc a, b
  sbc a, e
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x28);
        assert_hex_eq!(cpu.bcdn(), 1);
    }
    // }}}

    // {{{ test and_a_r8
    #[test]
    fn execute_and_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0xDE
  ld a, 0xFF
  ld [hl], b
  and a, [hl]
  and a, c
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x12);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 0);
    }
    // }}}

    // {{{ test xor_a_r8
    #[test]
    fn execute_xor_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0xDE
  ld a, 0xFF
  ld [hl], b
  xor a, [hl]
  xor a, c
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x32);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 0);
        assert_hex_eq!(cpu.carry(), 0);
    }
    // }}}

    // {{{ test or_a_r8
    #[test]
    fn execute_or_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0x5C
  ld a, 0x2A
  ld [hl], b
  or a, [hl]
  or a, c
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x7F);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 0);
        assert_hex_eq!(cpu.carry(), 0);
    }
    // }}}

    // {{{ test cp_a_r8
    #[test]
    fn execute_cp_a_r8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, 0xC000
  ld b, 0x5C
  ld a, 0x2A
  ld [hl], b
  cp a, a
  jr z, .skip
  inc a
.skip
  cp a, [hl]
  jr c, .skip2
  inc a
.skip2
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x2A);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test add_a_imm8
    #[test]
    fn execute_add_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  add a, 0xFF
    "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x00);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test adc_a_imm8
    #[test]
    fn execute_adc_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  adc a, 0xFF
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test sub_a_imm8
    #[test]
    fn execute_sub_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  sub a, 0xFF
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test sbc_a_imm8
    #[test]
    fn execute_sbc_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  sbc a, 0xFD
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x03);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 1);
    }
    // }}}

    // {{{ test and_a_imm8
    #[test]
    fn execute_and_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  and a, 0x05
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x05);
        assert_hex_eq!(cpu.zero(), 0);
        assert_hex_eq!(cpu.bcdn(), 0);
        assert_hex_eq!(cpu.bcdh(), 1);
        assert_hex_eq!(cpu.carry(), 0);
    }
    // }}}

    // {{{ test xor_a_imm8
    #[test]
    fn execute_xor_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  xor a, 0x06
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0xA3);
    }
    // }}}

    // {{{ test or_a_imm8
    #[test]
    fn execute_or_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  or a, 0x16
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0xB7);
    }
    // }}}

    // {{{ test cp_a_imm8
    #[test]
    fn execute_cp_a_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0x2A
  cp a, 0x2A
  jr z, .skip
  inc a
.skip
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x2A);
        assert_hex_eq!(cpu.zero(), 1);
        assert_hex_eq!(cpu.bcdn(), 1);
        assert_hex_eq!(cpu.bcdh(), 0);
        assert_hex_eq!(cpu.carry(), 0);
    }
    // }}}

    // {{{ test ret_cond
    #[test]
    fn execute_ret_cond() {
        const ROM: &[u8] = gbasm! {r#"
  call z, .foo
  inc a
  halt
.foo
  ret nz
  inc b
  ret nz
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.b(), 0x01);
        assert_hex_eq!(cpu.pc(), 0x155);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.ime(), 0);
    }
    // }}}

    // {{{ test ret
    #[test]
    fn execute_ret() {
        const ROM: &[u8] = gbasm! {r#"
  call z, .foo
  inc a
  halt
.foo
  ret
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.pc(), 0x155);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.ime(), 0);
    }
    // }}}

    // {{{ test reti
    #[test]
    fn execute_reti() {
        const ROM: &[u8] = gbasm! {r#"
  call z, .foo
  inc a
  halt
.foo
  reti
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.pc(), 0x155);
        assert_hex_eq!(cpu.sp(), 0xFFFE);
        assert_hex_eq!(cpu.ime(), 1);
    }
    // }}}

    // {{{ test jp_cond_imm16
    #[test]
    fn execute_jp_cond_imm16() {
        const ROM: &[u8] = gbasm! {r#"
jp z, Test1
  halt
Test1:
halt
jp c, Test2
halt
Test2:
jp nz, Test3
jp nc, Test4
halt
Test3:
inc a
Test4:
inc a
halt
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x01);
        assert_hex_eq!(cpu.pc(), 0x155);
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
        assert_hex_eq!(cpu.bc(), 0x0114);
        assert_hex_eq!(cpu.pc(), 0x0156);
        assert_hex_eq!(cpu.a(), 0x01);
    }
    // }}}

    // {{{ test jp_hl
    #[test]
    fn execute_jp_hl() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, .foo
  jp hl
  halt
.foo
  inc a
  halt
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.pc(), 0x0157);
    }
    // }}}

    // {{{ test call_cond_imm16
    #[test]
    fn execute_call_cond_imm16() {
        const ROM: &[u8] = gbasm! {r#"
  call nz,.foo
  inc a
  call c,.foo
  inc a
  inc a
  halt
.foo
  inc a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x03);
        assert_hex_eq!(cpu.sp(), 0xFFFC);
        assert_hex_eq!(cpu.pc(), 0x15C);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFD), 0x01);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFC), 0x57);
    }
    // }}}

    // {{{ test call_imm16
    #[test]
    fn execute_call_imm16() {
        const ROM: &[u8] = gbasm! {r#"
  call .foo
  halt
.foo
  inc a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x02);
        assert_hex_eq!(cpu.pc(), 0x0156);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFD), 0x01);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFC), 0x53);
    }
    // }}}

    // {{{ test rst_tgt3
    #[test]
    fn execute_rst_tgt3() {
        const ROM: &[u8] = gbasm! {r#"
  rst 0x18
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(7);
        assert_hex_eq!(cpu.pc(), 0x151);
        cpu.mtick(1);
        assert_hex_eq!(cpu.pc(), 0x18);
    }

    // {{{ test pop_r16stk
    #[test]
    #[ignore = "TODO"]
    fn execute_pop_r16stk() {
        const ROM: &[u8] = gbasm! {r#"
  ld bc, 0x1A2B
  push bc
  push bc
  push bc
  push bc
  ld bc, 0x0000
  pop bc
  pop de
  pop hl
  pop af
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.bc(), 0x1A2B);
        assert_hex_eq!(cpu.de(), 0x1A2B);
        assert_hex_eq!(cpu.hl(), 0x1A2B);
        assert_hex_eq!(cpu.af(), 0x1A2B);
    }
    // }}}

    // {{{ test push_r16stk
    #[test]
    fn execute_push_r16stk() {
        const ROM: &[u8] = gbasm! {r#"
  push bc
  push de
  push hl
  push af
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.af(), 0x01B0);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFD), 0x00);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFC), 0x13);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFB), 0x00);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFFA), 0xD8);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFF9), 0x01);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFF8), 0x4D);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFF7), 0x01);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFF6), 0xB0);
    }
    // }}}

    // {{{ test ldh_mc_a
    #[test]
    fn execute_ldh_mc_a() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ld c, 0x80
  ldh [c], a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.mem_dbg_read(0xFF80), 0xA5);
    }
    // }}}

    // {{{ test ldh_mimm8_a
    #[test]
    fn execute_ldh_mimm8_a() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ldh [0xFF80], a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.mem_dbg_read(0xFF80), 0xA5);
    }
    // }}}

    // {{{ test ld_mimm16_a
    #[test]
    fn execute_ld_mimm16_a() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ld [0xC000], a
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.mem_dbg_read(0xC000), 0xA5);
    }
    // }}}

    // {{{ test ldh_a_mc
    #[test]
    fn execute_ldh_a_mc() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ld c, 0x80
  ldh [c], a
  ld a, 0x00
  ldh a, [c]
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0xA5);
    }
    // }}}

    // {{{ test ldh_a_mimm8
    #[test]
    fn execute_ldh_a_mimm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ldh [0xFF80], a
  ld a, 0x00
  ldh a, [0xFF80]
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0xA5);
    }
    // }}}

    // {{{ test ld_a_mimm16
    #[test]
    fn execute_ld_a_mimm16() {
        const ROM: &[u8] = gbasm! {r#"
  ld a, 0xA5
  ld [0xC000], a
  ld a, 0x00
  ld a, [0xC000]
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.a(), 0x00);
    }
    // }}}

    // {{{ test add_sp_imm8
    #[test]
    fn execute_add_sp_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  add sp, -0x4
  push de
  add sp, 0x8
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.sp(), 0x0000);
        assert_hex_eq!(cpu.mem_dbg_read(0xFFF8), 0xD8);
    }
    // }}}

    // {{{ test ld_hl_sp_plus_imm8
    #[test]
    fn execute_ld_hl_sp_plus_imm8() {
        const ROM: &[u8] = gbasm! {r#"
  ld hl, sp-0x1
  ld b, h
  ld c, l
  ld hl, sp+0x1
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.bc(), 0xFFFD);
        assert_hex_eq!(cpu.hl(), 0xFFFF);
    }
    // }}}

    // {{{ test ld_sp_hl
    #[test]
    fn execute_ld_sp_hl() {
        const ROM: &[u8] = gbasm! {r#"
  ld sp, hl
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.sp(), 0x014D);
    }
    // }}}

    // {{{ test di
    #[test]
    fn execute_di() {
        const ROM: &[u8] = gbasm! {r#"
  ei
  di
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.ime(), 0);
    }
    // }}}

    // {{{ test ei
    #[test]
    fn execute_ei() {
        const ROM: &[u8] = gbasm! {r#"
  ei
        "#};
        let mut cpu = Cpu::init_dmg(ROM);
        cpu.mtick(200);
        assert_hex_eq!(cpu.ime(), 1);
    }
    // }}}

    // }}}
}
