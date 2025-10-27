use macros::*;
use std::fmt;

#[allow(dead_code)]
const M43: u8 = 0b00011000;
const M54: u8 = 0b00110000;
const M543: u8 = 0b00111000;
const M210: u8 = 0b00000111;

// {{{ Register Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R8 {
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
    A,
}

impl R8 {
    pub fn from(r8: u8) -> Self {
        match r8 {
            0 => R8::B,
            1 => R8::C,
            2 => R8::D,
            3 => R8::E,
            4 => R8::H,
            5 => R8::L,
            6 => R8::HL,
            7 => R8::A,
            _ => panic!("Invalid r8 operand: {:02x}", r8),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16 {
    BC,
    DE,
    HL,
    SP,
}

impl R16 {
    pub fn from(r16: u8) -> Self {
        match r16 {
            0 => R16::BC,
            1 => R16::DE,
            2 => R16::HL,
            3 => R16::SP,
            _ => panic!("Invalid r16 operand: {:02x}", r16),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16stk {
    BC,
    DE,
    HL,
    AF,
}

impl R16stk {
    pub fn from(r16: u8) -> Self {
        match r16 {
            0 => R16stk::BC,
            1 => R16stk::DE,
            2 => R16stk::HL,
            3 => R16stk::AF,
            _ => panic!("Invalid r16stk operand: {:02x}", r16),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R16mem {
    BC,
    DE,
    HLi,
    HLd,
}

impl R16mem {
    pub fn from(r16: u8) -> Self {
        match r16 {
            0 => R16mem::BC,
            1 => R16mem::DE,
            2 => R16mem::HLi,
            3 => R16mem::HLd,
            _ => panic!("Invalid r16 operand: {:02x}", r16),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cond {
    NZ,
    Z,
    NC,
    C,
}

impl Cond {
    pub fn from(cond: u8) -> Self {
        match cond {
            0 => Cond::NZ,
            1 => Cond::Z,
            2 => Cond::NC,
            3 => Cond::C,
            _ => panic!("Invalid cond operand: {:02x}", cond),
        }
    }
}

trait OverflowingAdd8 {
    fn halfcarry_add(self, rhs: u8) -> (u8, u8, u8, u8);
}

impl OverflowingAdd8 for u8 {
    fn halfcarry_add(self, rhs: u8) -> (u8, u8, u8, u8) {
        let (result, carry) = self.overflowing_add(rhs);
        let halfcarry = (self & 0x0F) + (rhs & 0x0F) > 0x0F;
        (
            result,
            if carry { 1 } else { 0 },
            if halfcarry { 1 } else { 0 },
            if result == 0 { 1 } else { 0 },
        )
    }
}

trait OverflowingSub8 {
    fn halfcarry_sub(self, rhs: u8) -> (u8, u8, u8, u8);
}

impl OverflowingSub8 for u8 {
    fn halfcarry_sub(self, rhs: u8) -> (u8, u8, u8, u8) {
        let (result, carry) = self.overflowing_sub(rhs);
        let halfcarry = (self & 0x0F) < (rhs & 0x0F);
        (
            result,
            if carry { 1 } else { 0 },
            if halfcarry { 1 } else { 0 },
            if result == 0 { 1 } else { 0 },
        )
    }
}

// }}}

// {{{ Cycle Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mc {
    M7,
    M6,
    M5,
    M4,
    M3,
    M2,
    M1,
    M0,
}
use Mc::*;

impl Mc {
    pub fn next(self) -> Self {
        use Mc::*;
        match self {
            M7 => M6,
            M6 => M5,
            M5 => M4,
            M4 => M3,
            M3 => M2,
            M2 => M1,
            M1 => panic!("Attempted to next M1"),
            M0 => panic!("Attempted to next M0"),
        }
    }
}
// }}}

pub struct Registers {
    ir: u8,
    ie: u8,
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    wz: u16,
}

pub struct Cpu {
    mem: [u8; 0xFFFF],
    m: u128,
    t: u128,
    r: Registers,
    addr: u16,
    data: u8,
    ime: u8,
    cb: u8,
    mc: Mc,
    executing: fn(&mut Cpu),
    halted: bool,
}

impl Cpu {
    // {{{ Execute Functions
    pub fn decode(&mut self) -> fn(&mut Cpu) {
        #[allow(clippy::manual_range_patterns)]
        if self.cb != 0 {
            self.cb = 0;
            match self.ir() {
                0x00..=0x07 => Cpu::rlc_r8,
                0x08..=0x0F => Cpu::rrc_r8,
                0x10..=0x17 => Cpu::rl_r8,
                0x18..=0x1F => Cpu::rr_r8,
                0x20..=0x27 => Cpu::sla_r8,
                0x28..=0x2F => Cpu::sra_r8,
                0x30..=0x37 => Cpu::swap_r8,
                0x38..=0x3F => Cpu::srl_r8,
                0x40..=0x7F => Cpu::bit_b3_r8,
                0x80..=0xBF => Cpu::res_b3_r8,
                0xC0..=0xFF => Cpu::set_b3_r8,
            }
        } else {
            match self.ir() {
                // Block 0
                0x00 => Cpu::nop,
                0x01 | 0x21 | 0x11 | 0x31 => Cpu::ld_r16_imm16,
                0x02 | 0x22 | 0x12 | 0x32 => Cpu::ld_mr16mem_a,
                0x0A | 0x2A | 0x1A | 0x3A => Cpu::ld_a_mr16mem,
                0x08 => Cpu::ld_mimm16_sp,
                //
                0x03 | 0x23 | 0x13 | 0x33 => Cpu::inc_r16,
                0x0B | 0x2B | 0x1B | 0x3B => Cpu::dec_r16,
                0x09 | 0x29 | 0x19 | 0x39 => Cpu::add_hl_r16,
                //
                0x04 | 0x24 | 0x14 | 0x0C | 0x2C | 0x1C | 0x3C => Cpu::inc_r8,
                0x34 => Cpu::inc_mhl,
                0x05 | 0x25 | 0x15 | 0x0D | 0x2D | 0x1D | 0x3D => Cpu::dec_r8,
                0x35 => Cpu::dec_mhl,
                //
                0x06 | 0x26 | 0x16 | 0x0E | 0x2E | 0x1E | 0x3E => Cpu::ld_r8_imm8,
                0x36 => Cpu::ld_mhl_imm8,
                //
                0x07 => Cpu::rlca,
                0x0F => Cpu::rrca,
                0x17 => Cpu::rla,
                0x1F => Cpu::rra,
                0x27 => Cpu::daa,
                0x2F => Cpu::cpl,
                0x37 => Cpu::scf,
                0x3F => Cpu::ccf,
                //
                0x18 => Cpu::jr_imm8,
                0x20 | 0x30 | 0x28 | 0x38 => Cpu::jr_cond_imm8,
                //
                0x10 => Cpu::stop,

                // Block 1
                0x40 | 0x60 | 0x50 | 0x48 | 0x68 | 0x58 | 0x78 | 0x44 | 0x64 | 0x54 | 0x4C
                | 0x6C | 0x5C | 0x7C | 0x42 | 0x62 | 0x52 | 0x4A | 0x6A | 0x5A | 0x7A | 0x41
                | 0x61 | 0x51 | 0x49 | 0x69 | 0x59 | 0x79 | 0x45 | 0x65 | 0x55 | 0x4D | 0x6D
                | 0x5D | 0x7D | 0x43 | 0x63 | 0x53 | 0x4B | 0x6B | 0x5B | 0x7B | 0x47 | 0x67
                | 0x57 | 0x4F | 0x6F | 0x5F | 0x7F => Cpu::ld_r8_r8,
                0x46 | 0x66 | 0x56 | 0x4E | 0x6E | 0x5E | 0x7E => Cpu::ld_r8_mhl,
                0x70 | 0x74 | 0x72 | 0x71 | 0x75 | 0x73 | 0x77 => Cpu::ld_mhl_r8,
                //
                0x76 => Cpu::halt,

                // Block 2
                0x80 | 0x84 | 0x82 | 0x81 | 0x85 | 0x83 | 0x86 | 0x87 => Cpu::add_a_r8,
                0x88 | 0x8C | 0x8A | 0x89 | 0x8D | 0x8B | 0x8E | 0x8F => Cpu::adc_a_r8,
                0x90 | 0x94 | 0x92 | 0x96 | 0x91 | 0x95 | 0x93 | 0x97 => Cpu::sub_a_r8,
                0x98 | 0x9C | 0x9A | 0x9E | 0x99 | 0x9D | 0x9B | 0x9F => Cpu::sbc_a_r8,
                0xA0 | 0xA4 | 0xA2 | 0xA6 | 0xA1 | 0xA5 | 0xA3 | 0xA7 => Cpu::and_a_r8,
                0xA8 | 0xAC | 0xAA | 0xAE | 0xA9 | 0xAD | 0xAB | 0xAF => Cpu::xor_a_r8,
                0xB0 | 0xB4 | 0xB2 | 0xB6 | 0xB1 | 0xB5 | 0xB3 | 0xB7 => Cpu::or_a_r8,
                0xB8 | 0xBC | 0xBA | 0xBE | 0xB9 | 0xBD | 0xBB | 0xBF => Cpu::cp_a_r8,

                // Block 3
                0xC6 => Cpu::add_a_imm8,
                0xCE => Cpu::adc_a_imm8,
                0xD6 => Cpu::sub_a_imm8,
                0xDE => Cpu::sbc_a_imm8,
                0xE6 => Cpu::and_a_imm8,
                0xEE => Cpu::xor_a_imm8,
                0xF6 => Cpu::or_a_imm8,
                0xFE => Cpu::cp_a_imm8,
                //
                0xC0 | 0xD0 | 0xC8 | 0xD8 => Cpu::ret_cond,
                0xC9 => Cpu::ret,
                0xD9 => Cpu::reti,
                0xC2 | 0xD2 | 0xCA | 0xDA => Cpu::jp_cond_imm16,
                0xC3 => Cpu::jp_imm16,
                0xE9 => Cpu::jp_hl,
                0xC4 | 0xD4 | 0xCC | 0xDC => Cpu::call_cond_imm16,
                0xCD => Cpu::call_imm16,
                0xC7 | 0xE7 | 0xD7 | 0xF7 | 0xCF | 0xEF | 0xDF | 0xFF => Cpu::rst_tgt3,
                0xC1 | 0xE1 | 0xD1 | 0xF1 => Cpu::pop_r16stk,
                0xC5 | 0xE5 | 0xD5 | 0xF5 => Cpu::push_r16stk,
                0xCB => Cpu::cb_prefix,
                0xE2 => Cpu::ldh_mc_a,
                0xE0 => Cpu::ldh_mimm8_a,
                0xEA => Cpu::ld_mimm16_a,
                0xF2 => Cpu::ldh_a_mc,
                0xF0 => Cpu::ldh_a_mimm8,
                0xFA => Cpu::ld_a_mimm16,
                0xE8 => Cpu::add_sp_imm8,
                0xF8 => Cpu::ld_hl_sp_plus_imm8,
                0xF9 => Cpu::ld_sp_hl,
                0xF3 => Cpu::di,
                0xFB => Cpu::ei,
                _ => panic!("Opcode not implemented: 0x{:02x}", self.ir()),
            }
        }
    }

    pub fn execute(&mut self) {
        if self.pc() >= 0x150 {
            eprintln!("Execute Invoked!\n{}", self);
        }
        (self.executing)(self);
        self.mc = self.mc.next();
        if self.pc() >= 0x150 {
            eprintln!("Execute Resolved!\n{}", self);
        }
    }

    pub fn fetch_next(&mut self) {
        self.addr = self.pc();
        self.mem_read();
        self.set_ir(self.data);
        self.inc_pc();
        self.mc = M0;
        self.executing = self.decode();
        (self.executing)(self);
    }

    pub fn fetch_next_addr(&mut self, addr: u16) {
        self.set_pc(addr);
        self.fetch_next();
    }

    pub fn pop_imm8_into_z(&mut self) {
        self.addr = self.pc();
        self.mem_read();
        self.set_z(self.data);
        self.inc_pc();
    }

    pub fn mask_bit(&self) -> u8 {
        1 << ((self.ir() & M543) >> 3)
    }

    pub fn init_dmg(cartridge: &[u8]) -> Self {
        let mut mem = [0u8; 0xFFFF];
        mem[0x0000..cartridge.len()].copy_from_slice(cartridge);

        let r = Registers {
            ir: 0x00,
            ie: 0x00,
            af: 0x01B0,
            bc: 0x0013,
            de: 0x00d8,
            hl: 0x014d,
            sp: 0xfffe,
            pc: 0x0100,
            wz: 0x0000, // ???
        };
        Cpu {
            mem,
            m: 0,
            t: 0,
            r,
            addr: 0x0000,
            data: 0x0000,
            ime: 0,
            cb: 0,
            mc: Mc::M1,
            executing: Cpu::nop,
            halted: false,
        }
    }

    // {{{ opcode nop
    pub fn nop(&mut self) {
        match self.mc {
            Mc::M1 => self.fetch_next(),
            Mc::M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_r16_imm16
    pub fn ld_r16_imm16(&mut self) {
        let r16 = R16::from((self.ir() & M54) >> 4);
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M1 => {
                self.set_r16(r16, self.wz());
                self.fetch_next()
            }
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_mr16mem_a
    pub fn ld_mr16mem_a(&mut self) {
        match self.mc {
            M2 => {
                let r16mem = R16mem::from((self.ir() & M54) >> 4);
                self.addr = self.r16mem(r16mem);
                self.data = self.a();
                self.mem_write();

                match r16mem {
                    R16mem::HLi => self.set_hl(self.hl() + 1),
                    R16mem::HLd => self.set_hl(self.hl() - 1),
                    _ => (),
                }
            }
            M1 => self.fetch_next(),
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_a_mr16mem
    pub fn ld_a_mr16mem(&mut self) {
        match self.mc {
            M2 => {
                let r16mem = R16mem::from((self.ir() & M54) >> 4);
                self.addr = self.r16mem(r16mem);
                self.mem_read();
                self.set_z(self.data);

                match r16mem {
                    R16mem::HLi => self.set_hl(self.hl() + 1),
                    R16mem::HLd => self.set_hl(self.hl() - 1),
                    _ => (),
                }
            }
            M1 => {
                self.set_a(self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_mimm16_sp
    pub fn ld_mimm16_sp(&mut self) {
        match self.mc {
            M5 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M3 => {
                self.addr = self.wz();
                self.data = self.lo(R16::SP);
                self.mem_write();
                self.set_wz(self.wz() + 1);
            }
            M2 => {
                self.addr = self.wz();
                self.data = self.hi(R16::SP);
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M6),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode inc_r16
    pub fn inc_r16(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.r16(R16::from((self.ir() & M54) >> 4));
            }
            M1 => {
                let r16 = R16::from((self.ir() & M54) >> 4);

                let result = self.r16(r16).wrapping_add(1);
                eprintln!("Incrementing {:?} to {:04x}", r16, result);

                self.set_r16(r16, result);
                self.fetch_next()
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode dec_r16
    pub fn dec_r16(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.r16(R16::from((self.ir() & M54) >> 4));
            }
            M1 => {
                let r16 = R16::from((self.ir() & M54) >> 4);

                let result = self.r16(r16).wrapping_sub(1);
                eprintln!("Decrementing {:?} to {:04x}", r16, result);

                self.set_r16(r16, result);
                self.fetch_next()
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode add_hl_r16
    pub fn add_hl_r16(&mut self) {
        match self.mc {
            M2 => {
                self.addr = 0x0000;
                let r16 = R16::from((self.ir() & M54) >> 4);
                let (r, c, h, _) = self.l().halfcarry_add(self.lo(r16));
                eprintln!(
                    "Adding {:?} lo ({:02x}) to HL => {:02x}",
                    r16,
                    self.lo(r16),
                    r
                );

                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_carry(c);

                self.set_l(r);
            }
            M1 => {
                let r16 = R16::from((self.ir() & M54) >> 4);

                let (r1, c1, h1, _) = self.h().halfcarry_add(self.hi(r16));
                let (r, c2, h2, _) = r1.halfcarry_add(self.carry());
                eprintln!(
                    "Adding {:?} hi ({:02x}) to HL => {:02x}",
                    r16,
                    self.hi(r16),
                    r
                );

                self.set_bcdn(0);
                if c1 + c2 > 0 {
                    self.set_carry(1);
                } else {
                    self.set_carry(0);
                }
                if h1 + h2 > 0 {
                    self.set_bcdh(1);
                } else {
                    self.set_bcdh(0);
                }

                self.set_h(r);
                self.fetch_next()
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode inc_r8
    pub fn inc_r8(&mut self) {
        match self.mc {
            M1 => {
                let r8 = R8::from((self.ir() & M543) >> 3);
                let (result, _, h, z) = self.r8(r8).halfcarry_add(1);

                eprintln!("Incrementing {:?} to {:02x}", r8, result);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_r8(r8, result);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode inc_mhl
    pub fn inc_mhl(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                let (result, _, h, z) = self.z().halfcarry_add(1);

                eprintln!("Incrementing [{:?}] to {:02x}", self.hl(), result);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h);

                self.data = result;
                self.mem_write();
            }
            M1 => self.fetch_next(),
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode dec_r8
    pub fn dec_r8(&mut self) {
        match self.mc {
            M1 => {
                let r8 = R8::from((self.ir() & M543) >> 3);
                eprintln!("dec_r8 r8:{:?}", r8);

                let (result, _, h, z) = self.r8(r8).halfcarry_sub(1);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);
                self.set_r8(r8, result);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode inc_mhl
    pub fn dec_mhl(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                let (result, _, h, z) = self.z().halfcarry_sub(1);

                eprintln!("Decrementing [{:?}] to {:02x}", self.hl(), result);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);

                self.data = result;
                self.mem_write();
            }
            M1 => self.fetch_next(),
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode_ld_r8_imm8
    pub fn ld_r8_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let r8 = R8::from((self.ir() & M543) >> 3);
                self.set_r8(r8, self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode_ld_mhl_imm8
    pub fn ld_mhl_imm8(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = self.hl();
                self.data = self.z();
                self.mem_write();
            }
            M1 => self.fetch_next(),
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rlca
    pub fn rlca(&mut self) {
        match self.mc {
            M1 => {
                self.set_carry((self.a() & 0x80) >> 7);
                self.set_a(self.a().rotate_left(1));

                self.set_zero(0);
                self.set_bcdn(0);
                self.set_bcdh(0);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }

    // {{{ opcode rrca
    pub fn rrca(&mut self) {
        match self.mc {
            M1 => {
                self.set_carry(self.a() & 0x01);
                self.set_a(self.a().rotate_right(1));

                self.set_zero(0);
                self.set_bcdn(0);
                self.set_bcdh(0);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rla
    pub fn rla(&mut self) {
        match self.mc {
            M1 => {
                let c = self.carry();
                self.set_carry((self.a() & 0x80) >> 7);
                self.set_a((self.a() << 1) | c);

                self.set_zero(0);
                self.set_bcdn(0);
                self.set_bcdh(0);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rra
    pub fn rra(&mut self) {
        match self.mc {
            M1 => {
                let c = self.carry();
                self.set_carry(self.a() & 0x01);
                self.set_a((self.a() >> 1) | (c << 7));

                self.set_zero(0);
                self.set_bcdn(0);
                self.set_bcdh(0);

                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode daa
    pub fn daa(&mut self) {
        match self.mc {
            M1 => {
                self.fetch_next();
                todo!("Opcode {} unimplemented", function!());
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode cpl
    pub fn cpl(&mut self) {
        match self.mc {
            M1 => {
                self.set_a(self.a() ^ 0xFF);
                self.set_bcdn(1);
                self.set_bcdh(1);
                self.fetch_next();
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode scf
    pub fn scf(&mut self) {
        match self.mc {
            M1 => {
                self.set_carry(1);
                self.fetch_next();
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ccf
    pub fn ccf(&mut self) {
        match self.mc {
            M1 => {
                self.set_carry(self.carry() ^ 1);
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.fetch_next();
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode jr_imm8
    pub fn jr_imm8(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                // ??? are the lower 8 bits of addr ignored by IDU here?
                self.addr = (self.pch() as u16) << 8;
                let zsign = self.z() >> 7 == 0x01;
                let (r, c) = self.z().overflowing_add(self.pcl());
                self.set_z(r);
                self.data = r;
                let w = if c && !zsign {
                    self.pch() + 1
                } else if !c && zsign {
                    self.pch() - 1
                } else {
                    self.pch()
                };
                self.set_w(w);
            }
            M0 => self.set_mc(M4),
            M1 => self.fetch_next_addr(self.wz()),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode jr_cond_imm8
    pub fn jr_cond_imm8(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                if self.cond() {
                    // ??? are the lower 8 bits of addr ignored by IDU here?
                    self.addr = (self.pch() as u16) << 8;
                    let zsign = self.z() >> 7 == 0x01;
                    let (r, c) = self.z().overflowing_add(self.pcl());
                    self.set_z(r);
                    self.data = r;
                    let w = if c && !zsign {
                        self.pch() + 1
                    } else if !c && zsign {
                        self.pch() - 1
                    } else {
                        self.pch()
                    };
                    self.set_w(w);
                } else {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_z(self.data());
                    self.inc_pc();
                }
            }
            M1 => {
                if self.cond() {
                    self.fetch_next_addr(self.wz());
                } else {
                    self.fetch_next();
                }
            }
            M0 => {
                if self.cond() {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M3)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode stop
    pub fn stop(&mut self) {
        match self.mc {
            M1 => {
                self.fetch_next();
                todo!("Opcode {} unimplemented", function!());
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // Block 1

    // {{{ opcode ld_r8_r8
    pub fn ld_r8_r8(&mut self) {
        match self.mc {
            M1 => {
                let r8_source = self.r8_operand();
                let r8_dest = R8::from((self.ir() & M543) >> 3);
                self.set_r8(r8_dest, self.r8(r8_source));
                self.fetch_next();
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_r8_mhl
    pub fn ld_r8_mhl(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_dest = R8::from((self.ir() & M543) >> 3);
                self.set_r8(r8_dest, self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_mhl_r8
    pub fn ld_mhl_r8(&mut self) {
        match self.mc {
            M2 => {
                let r8_source = self.r8_operand();
                self.addr = self.hl();
                self.data = self.r8(r8_source);
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode halt
    pub fn halt(&mut self) {
        match self.mc {
            M1 => {
                self.halted = true;
                // TODO: halt has a lot of interactions to implement still
                // self.fetch_next()
                self.set_mc(M2);
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // Block 2

    // {{{ opcode add_a_r8
    pub fn add_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let (r, c, h, z) = self.a().halfcarry_add(match r8_operand {
                    R8::HL => self.z(),
                    _ => self.r8(r8_operand),
                });
                self.set_a(r);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode adc_a_r8
    pub fn adc_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let (r1, c1, h1, _) = self.a().halfcarry_add(match r8_operand {
                    R8::HL => self.z(),
                    _ => self.r8(r8_operand),
                });
                let (r2, c2, h2, z) = r1.halfcarry_add(self.carry());
                self.set_a(r2);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h1 | h2);
                self.set_carry(c1 | c2);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sub_a_r8
    pub fn sub_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let (r, c, h, z) = self.a().halfcarry_sub(match r8_operand {
                    R8::HL => self.z(),
                    _ => self.r8(r8_operand),
                });
                self.set_a(r);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sbc_a_r8
    pub fn sbc_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let (r1, c1, h1, _) = self.a().halfcarry_sub(match r8_operand {
                    R8::HL => self.z(),
                    _ => self.r8(r8_operand),
                });
                let (r2, c2, h2, z) = r1.halfcarry_sub(self.carry());
                self.set_a(r2);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h1 | h2);
                self.set_carry(c1 | c2);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode and_a_r8
    pub fn and_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let r = self.a()
                    & match r8_operand {
                        R8::HL => self.z(),
                        _ => self.r8(r8_operand),
                    };
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(1);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode xor_a_r8
    pub fn xor_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let r = self.a()
                    ^ match r8_operand {
                        R8::HL => self.z(),
                        _ => self.r8(r8_operand),
                    };
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode or_a_r8
    pub fn or_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let r = self.a()
                    | match r8_operand {
                        R8::HL => self.z(),
                        _ => self.r8(r8_operand),
                    };
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode cp_a_r8
    pub fn cp_a_r8(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M1 => {
                let r8_operand = self.r8_operand();
                let (_, c, h, z) = self.a().halfcarry_sub(match r8_operand {
                    R8::HL => self.z(),
                    _ => self.r8(r8_operand),
                });

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => match self.r8_operand() {
                R8::HL => self.set_mc(M3),
                _ => self.set_mc(M2),
            },
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // Block 3

    // {{{ opcode add_a_imm8
    pub fn add_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let (r, c, h, z) = self.a().halfcarry_add(self.z());
                self.set_a(r);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode adc_a_imm8
    pub fn adc_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let (r1, c1, h1, _) = self.a().halfcarry_add(self.z());
                let (r2, c2, h2, z) = r1.halfcarry_add(self.carry());
                self.set_a(r2);

                self.set_zero(z);
                self.set_bcdn(0);
                self.set_bcdh(h1 | h2);
                self.set_carry(c1 | c2);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sub_a_imm8
    pub fn sub_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let (r, c, h, z) = self.a().halfcarry_sub(self.z());
                self.set_a(r);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sbc_a_imm8
    pub fn sbc_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let (r1, c1, h1, _) = self.a().halfcarry_sub(self.z());
                let (r2, c2, h2, z) = r1.halfcarry_sub(self.carry());
                self.set_a(r2);

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h1 | h2);
                self.set_carry(c1 | c2);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode and_a_imm8
    pub fn and_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let r = self.a() & self.z();
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(1);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode xor_a_imm8
    pub fn xor_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let r = self.a() ^ self.z();
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode or_a_imm8
    pub fn or_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let r = self.a() | self.z();
                self.set_a(r);

                if r == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.set_carry(0);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode cp_a_imm8
    pub fn cp_a_imm8(&mut self) {
        match self.mc {
            M2 => {
                self.pop_imm8_into_z();
            }
            M1 => {
                let (_, c, h, z) = self.a().halfcarry_sub(self.z());

                self.set_zero(z);
                self.set_bcdn(1);
                self.set_bcdh(h);
                self.set_carry(c);

                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ret_cond
    pub fn ret_cond(&mut self) {
        match self.mc {
            M5 => self.addr = 0x0000,
            M4 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_z(self.data);
                self.inc_sp();
            }
            M3 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_w(self.data);
                self.inc_sp();
            }
            M2 => {
                if self.cond() {
                    self.addr = 0x0000;
                    self.set_pc(self.wz());
                } else {
                    self.addr = 0x0000;
                }
            }
            M1 => {
                self.fetch_next();
            }
            M0 => {
                if self.cond() {
                    self.set_mc(M6)
                } else {
                    self.set_mc(M3)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ret
    pub fn ret(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_z(self.data);
                self.inc_sp();
            }
            M3 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_w(self.data);
                self.inc_sp();
            }
            M2 => {
                self.addr = 0x0000;
                self.set_pc(self.wz());
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode reti
    pub fn reti(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_z(self.data);
                self.inc_sp();
            }
            M3 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_w(self.data);
                self.inc_sp();
            }
            M2 => {
                self.addr = 0x0000;
                self.set_pc(self.wz());
                self.set_ime(1);
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode jp_cond_imm16
    pub fn jp_cond_imm16(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M3 => {
                if self.cond() {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_w(self.data);
                    self.inc_pc();
                } else {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_z(self.data);
                    self.inc_pc();
                }
            }
            M2 => {
                if self.cond() {
                    self.addr = 0x0000;
                    self.set_pc(self.wz());
                } else {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_w(self.data);
                    self.inc_pc();
                }
            }
            M1 => {
                self.fetch_next();
            }
            M0 => {
                if self.cond() {
                    self.set_mc(M5)
                } else {
                    self.set_mc(M4)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode jp_imm16
    pub fn jp_imm16(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = 0x0000;
                self.set_pc(self.wz());
            }
            M1 => self.fetch_next(),
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }

    // {{{ opcode jp_hl
    pub fn jp_hl(&mut self) {
        match self.mc {
            M1 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_ir(self.data());
                self.set_pc(self.hl() + 1);
                self.set_mc(M0);
                self.executing = self.decode();
                (self.executing)(self);
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode call_cond_imm16
    pub fn call_cond_imm16(&mut self) {
        match self.mc {
            M6 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M5 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M4 => self.dec_sp(),
            M3 => {
                if self.cond() {
                    self.set_addr(self.sp());
                    self.data = self.pch();
                    self.mem_write();
                    self.dec_sp();
                } else {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_z(self.data);
                    self.inc_pc();
                }
            }
            M2 => {
                if self.cond() {
                    self.set_addr(self.sp());
                    self.data = self.pcl();
                    self.mem_write();
                    self.set_pc(self.wz());
                } else {
                    self.addr = self.pc();
                    self.mem_read();
                    self.set_w(self.data);
                    self.inc_pc();
                }
            }
            M1 => {
                self.fetch_next();
            }
            M0 => {
                if self.cond() {
                    self.set_mc(M7)
                } else {
                    self.set_mc(M4)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode call_imm16
    pub fn call_imm16(&mut self) {
        match self.mc {
            M6 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M5 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M4 => self.dec_sp(),
            M3 => {
                self.set_addr(self.sp());
                self.data = self.pch();
                self.mem_write();
                self.dec_sp();
            }
            M2 => {
                self.set_addr(self.sp());
                self.data = self.pcl();
                self.mem_write();
                self.set_pc(self.wz());
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M7),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rst_tgt3
    pub fn rst_tgt3(&mut self) {
        match self.mc {
            M4 => self.dec_sp(),
            M3 => {
                self.set_addr(self.sp());
                self.data = self.pch();
                self.mem_write();
                self.dec_sp();
            }
            M2 => {
                self.set_addr(self.sp());
                self.data = self.pcl();
                self.mem_write();
                self.set_pc((self.ir() & M543) as u16);
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode pop_r16stk
    pub fn pop_r16stk(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_z(self.data);
                self.inc_sp();
            }
            M2 => {
                self.addr = self.sp();
                self.mem_read();
                self.set_w(self.data);
                self.inc_sp();
            }
            M1 => {
                let r16stk = R16stk::from((self.ir() & M54) >> 4);
                self.set_r16stk(r16stk, self.wz());
                self.fetch_next();
            }
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode push_r16stk
    pub fn push_r16stk(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.sp();
                self.dec_sp();
            }
            M3 => {
                self.addr = self.sp();
                let r16stk = R16stk::from((self.ir() & M54) >> 4);
                self.data = (self.r16stk(r16stk) >> 8) as u8;
                self.mem_write();
                self.dec_sp();
            }
            M2 => {
                self.addr = self.sp();
                let r16stk = R16stk::from((self.ir() & M54) >> 4);
                self.data = self.r16stk(r16stk) as u8;
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode cg_prefix
    pub fn cb_prefix(&mut self) {
        match self.mc {
            M1 => {
                self.cb = 1;
                self.fetch_next();
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ldh_mc_a
    pub fn ldh_mc_a(&mut self) {
        match self.mc {
            M2 => {
                self.addr = 0xFF00 + self.c() as u16;
                self.data = self.a();
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ldh_mimm8_a
    pub fn ldh_mimm8_a(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = 0xFF00 + self.z() as u16;
                self.data = self.a();
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_mimm16_a
    pub fn ld_mimm16_a(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = self.wz();
                self.data = self.a();
                self.mem_write();
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ldh_a_mc
    pub fn ldh_a_mc(&mut self) {
        match self.mc {
            M2 => {
                self.addr = 0xFF00 + self.c() as u16;
                self.mem_read();
                self.set_z(self.data);
            }
            M1 => {
                self.set_a(self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ldh_a_mimm8
    pub fn ldh_a_mimm8(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = 0xFF00 + self.z() as u16;
                self.mem_read();
                self.set_z(self.data);
            }
            M1 => {
                self.set_a(self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_a_mimm16
    pub fn ld_a_mimm16(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_w(self.data);
                self.inc_pc();
            }
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = 0xFF00 + self.z() as u16;
                self.mem_read();
                self.set_z(self.data);
            }
            M1 => {
                self.set_a(self.z());
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode add_sp_imm8
    pub fn add_sp_imm8(&mut self) {
        match self.mc {
            M4 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M3 => {
                self.addr = 0x0000;
                let (r, c, h, _) = if self.z() & 0x80 != 0 {
                    self.spl().halfcarry_sub(!(self.z()) + 0x1)
                } else {
                    self.spl().halfcarry_add(self.z())
                };
                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_carry(c);

                self.set_z(r);
                self.data = self.z();
            }
            M2 => {
                self.addr = 0x0000;
                let (r, _, _, _) = self.sph().halfcarry_add(self.carry());
                self.set_w(r);
                self.data = self.w();
            }
            M1 => {
                self.set_sp(self.wz());
                self.fetch_next();
            }
            M0 => self.set_mc(M5),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_hl_sp_plus_imm8
    pub fn ld_hl_sp_plus_imm8(&mut self) {
        match self.mc {
            M3 => {
                self.addr = self.pc();
                self.mem_read();
                self.set_z(self.data);
                self.inc_pc();
            }
            M2 => {
                self.addr = 0x0000;
                let (r, c, h, _) = if self.z() & 0x80 != 0 {
                    self.spl().halfcarry_sub(!(self.z()) + 0x1)
                } else {
                    self.spl().halfcarry_add(self.z())
                };
                self.set_bcdn(0);
                self.set_bcdh(h);
                self.set_carry(c);

                self.set_l(r);
                self.data = self.z();
            }
            M1 => {
                let (r, _, _, _) = self.sph().halfcarry_add(self.carry());
                self.set_h(r);
                self.fetch_next();
            }
            M0 => self.set_mc(M4),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ld_sp_hl
    pub fn ld_sp_hl(&mut self) {
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.set_sp(self.hl());
            }
            M1 => {
                self.fetch_next();
            }
            M0 => self.set_mc(M3),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode di
    pub fn di(&mut self) {
        match self.mc {
            M1 => {
                self.fetch_next();
                self.set_ime(0);
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode ei
    pub fn ei(&mut self) {
        match self.mc {
            M1 => {
                self.fetch_next();
                self.set_ime(1);
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rlc_r8
    pub fn rlc_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                self.set_carry((self.z() & 0x80) >> 7);
                self.addr = self.hl();
                self.data = self.z().rotate_left(1);
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_carry((self.r8(r8) & 0x80) >> 7);
                    self.set_r8(r8, self.r8(r8).rotate_left(1));

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rrc_r8
    pub fn rrc_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                self.set_carry(self.z() & 0x01);
                self.addr = self.hl();
                self.data = self.z().rotate_right(1);
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_carry(self.r8(r8) & 0x01);
                    self.set_r8(r8, self.r8(r8).rotate_right(1));

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rl_r8
    pub fn rl_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                let c = self.carry();
                self.set_carry((self.z() & 0x80) >> 7);
                self.addr = self.hl();
                self.data = (self.z() << 1) | c;
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    let c = self.carry();
                    self.set_carry((self.r8(r8) & 0x80) >> 7);
                    self.set_r8(r8, (self.r8(r8) << 1) | c);

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode rr_r8
    pub fn rr_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                let c = self.carry();
                self.set_carry(self.z() & 0x1);
                self.addr = self.hl();
                self.data = (self.z() >> 1) | (c << 7);
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    let c = self.carry();
                    self.set_carry(self.r8(r8) & 0x01);
                    self.set_r8(r8, (self.r8(r8) >> 1) | (c << 7));

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sla_r8
    pub fn sla_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                self.set_carry((self.z() & 0x80) >> 7);
                self.addr = self.hl();
                self.data = self.z() << 1;
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_carry((self.r8(r8) & 0x80) >> 7);
                    self.set_r8(r8, self.r8(r8) << 1);

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode sra_r8
    pub fn sra_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                let hi = self.z() & 0x80;
                self.set_carry(self.z() & 0x1);
                self.addr = self.hl();
                self.data = (self.z() >> 1) | hi;
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    let hi = self.r8(r8) & 0x80;
                    self.set_carry(self.r8(r8) & 0x01);
                    self.set_r8(r8, (self.r8(r8) >> 1) | hi);

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode swap_r8
    pub fn swap_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                self.addr = self.hl();
                self.data = self.z().rotate_right(4);
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
                self.set_carry(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_r8(r8, self.r8(r8).rotate_right(4));

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.set_carry(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode srl_r8
    pub fn srl_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data());
            }
            M2 => {
                self.set_carry(self.z() & 0x1);
                self.addr = self.hl();
                self.data = self.z() >> 1;
                self.mem_write();

                if self.data == 0 {
                    self.set_zero(1);
                } else {
                    self.set_zero(0);
                }
                self.set_bcdn(0);
                self.set_bcdh(0);
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_carry(self.r8(r8) & 0x01);
                    self.set_r8(r8, self.r8(r8) >> 1);

                    if self.r8(r8) == 0 {
                        self.set_zero(1);
                    } else {
                        self.set_zero(0);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(0);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode bit_b3_r8
    pub fn bit_b3_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M2 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data);
            }
            M1 => {
                if r8 == R8::HL {
                    let bit = self.mask_bit();
                    if self.z() & bit == bit {
                        self.set_zero(0);
                    } else {
                        self.set_zero(1);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(1);
                    self.fetch_next();
                } else {
                    let bit = self.mask_bit();
                    if self.r8(r8) & bit == bit {
                        self.set_zero(0);
                    } else {
                        self.set_zero(1);
                    }
                    self.set_bcdn(0);
                    self.set_bcdh(1);
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M3)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode res_b3_r8
    pub fn res_b3_r8(&mut self) {
        let r8 = self.r8_operand();
        match self.mc {
            M3 => {
                self.addr = self.hl();
                self.mem_read();
                self.set_z(self.data);
            }
            M2 => {
                self.addr = self.hl();
                self.data = self.z() & !self.mask_bit();
                self.mem_write();
            }
            M1 => {
                if r8 == R8::HL {
                    self.fetch_next();
                } else {
                    self.set_r8(r8, self.r8(r8) & !self.mask_bit());
                    self.fetch_next();
                }
            }
            M0 => {
                if r8 == R8::HL {
                    self.set_mc(M4)
                } else {
                    self.set_mc(M2)
                }
            }
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // {{{ opcode set_b3_r8
    pub fn set_b3_r8(&mut self) {
        match self.mc {
            M1 => {
                self.fetch_next();
                todo!("Opcode {} unimplemented", function!());
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in {}: {:?}", function!(), self.mc),
        }
    }
    // }}}

    // }}} end Execute Functions

    // {{{ Cycle Functions
    pub fn tick_t1(&mut self) {
        self.t += 1;
        if self.t.is_multiple_of(4) {
            self.m += 1;

            if !self.halted {
                self.execute();
            }
        }
    }

    pub fn tick4(&mut self) {
        for _ in 0..4 {
            self.tick_t1()
        }
    }

    pub fn mtick(&mut self, mcycles: usize) {
        for _ in 0..mcycles {
            self.tick4();
        }
    }
    // }}}

    // {{{ Memory Functions
    pub fn mem_read(&mut self) {
        self.data = self.mem[self.addr as usize];
    }

    pub fn mem_dbg_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn mem_write(&mut self) {
        let addr = self.addr();
        let data = self.data();
        match addr {
            0x0000..0x4000 => todo!("Memory write to ROM bank 00: {:04x}:{:02x}", addr, data),
            0x4000..0x8000 => todo!("Memory write to ROM bank 01-NN: {:04x}:{:02x}", addr, data),
            0x8000..0xA000 => self.mem[addr as usize] = data, // 8 KiB VRAM (GBC Bank 00-01)
            0xA000..0xC000 => self.mem[addr as usize] = data, // 8 KiB External RAM
            0xC000..0xD000 => self.mem[addr as usize] = data, // 4 KiB Work RAM
            0xD000..0xE000 => self.mem[addr as usize] = data, // 4 KiB Work RAM (GBC Bank 01-07)
            0xE000..0xFE00 => panic!("Memory write to echo RAM: {:04x}:{:02x}", addr, data),
            0xFE00..0xFEA0 => todo!("Memory write to OAM: {:04x}:{:02x}", addr, data),
            0xFEA0..0xFF00 => panic!("Memory write to not usable: {:04x}:{:02x}", addr, data),
            0xFF00..0xFF80 => todo!("Memory write to I/O registers: {:04x}:{:02x}", addr, data),
            0xFF80..0xFFFF => self.mem[addr as usize] = data, // High RAM (HRAM)
            0xFFFF => todo!("Memory write to IE register: {:04x}:{:02x}", addr, data),
        }
    }

    pub fn mem_dbg_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data
    }

    pub fn mem_bulk_write(&mut self, addr: u16, mem: &[u8]) {
        self.mem[addr as usize..mem.len()].copy_from_slice(mem);
    }
    // }}}

    // {{{ CPU Getters
    pub fn r8(&self, r8: R8) -> u8 {
        match r8 {
            R8::B => self.b(),
            R8::C => self.c(),
            R8::D => self.d(),
            R8::E => self.e(),
            R8::H => self.h(),
            R8::L => self.l(),
            R8::HL => todo!(
                "Tried to retrieve [hl]: {:04x}:{:02x}",
                self.hl(),
                self.mem_dbg_read(self.hl())
            ),
            R8::A => self.a(),
        }
    }

    pub fn r8_operand(&self) -> R8 {
        R8::from(self.ir() & M210)
    }

    pub fn r16(&self, r16: R16) -> u16 {
        match r16 {
            R16::BC => self.bc(),
            R16::DE => self.de(),
            R16::HL => self.hl(),
            R16::SP => self.sp(),
        }
    }

    pub fn r16stk(&self, r16stk: R16stk) -> u16 {
        match r16stk {
            R16stk::BC => self.bc(),
            R16stk::DE => self.de(),
            R16stk::HL => self.hl(),
            R16stk::AF => self.af(),
        }
    }

    pub fn r16mem(&self, r16mem: R16mem) -> u16 {
        match r16mem {
            R16mem::BC => self.bc(),
            R16mem::DE => self.de(),
            R16mem::HLi => self.hl(),
            R16mem::HLd => self.hl(),
        }
    }

    pub fn hi(&self, r16: R16) -> u8 {
        match r16 {
            R16::BC => self.b(),
            R16::DE => self.d(),
            R16::HL => self.h(),
            R16::SP => ((self.sp() & 0xFF00) >> 8) as u8,
        }
    }

    pub fn lo(&self, r16: R16) -> u8 {
        match r16 {
            R16::BC => self.c(),
            R16::DE => self.e(),
            R16::HL => self.l(),
            R16::SP => (self.sp() & 0x00FF) as u8,
        }
    }

    pub fn m(&self) -> u128 {
        self.m
    }

    pub fn t(&self) -> u128 {
        self.t
    }

    pub fn mc(&self) -> Mc {
        self.mc
    }

    pub fn addr(&self) -> u16 {
        self.addr
    }

    pub fn data(&self) -> u8 {
        self.data
    }

    pub fn ime(&self) -> u8 {
        self.ime
    }

    pub fn af(&self) -> u16 {
        self.r.af
    }

    pub fn ir(&self) -> u8 {
        self.r.ir
    }

    pub fn ie(&self) -> u8 {
        self.r.ie
    }

    pub fn a(&self) -> u8 {
        ((self.r.af & 0xFF00) >> 8) as u8
    }

    pub fn f(&self) -> u8 {
        self.r.af as u8
    }

    pub fn zero(&self) -> u8 {
        ((self.r.af & 0x80) >> 7) as u8
    }

    pub fn bcdn(&self) -> u8 {
        ((self.r.af & 0x40) >> 6) as u8
    }

    pub fn bcdh(&self) -> u8 {
        ((self.r.af & 0x20) >> 5) as u8
    }

    pub fn carry(&self) -> u8 {
        ((self.r.af & 0x10) >> 4) as u8
    }

    pub fn cond(&self) -> bool {
        let res = match Cond::from((self.ir() & M43) >> 3) {
            Cond::NZ => self.zero() == 0,
            Cond::Z => self.zero() == 1,
            Cond::NC => self.carry() == 0,
            Cond::C => self.carry() == 1,
        };
        eprintln!("COND IS {}", res);
        res
    }

    pub fn bc(&self) -> u16 {
        self.r.bc
    }

    pub fn b(&self) -> u8 {
        ((self.r.bc & 0xFF00) >> 8) as u8
    }

    pub fn c(&self) -> u8 {
        self.r.bc as u8
    }

    pub fn de(&self) -> u16 {
        self.r.de
    }

    pub fn d(&self) -> u8 {
        ((self.r.de & 0xFF00) >> 8) as u8
    }

    pub fn e(&self) -> u8 {
        self.r.de as u8
    }

    pub fn hl(&self) -> u16 {
        self.r.hl
    }

    pub fn h(&self) -> u8 {
        ((self.r.hl & 0xFF00) >> 8) as u8
    }

    pub fn l(&self) -> u8 {
        self.r.hl as u8
    }

    pub fn sp(&self) -> u16 {
        self.r.sp
    }

    pub fn spl(&self) -> u8 {
        (self.r.sp & 0xFF) as u8
    }

    pub fn sph(&self) -> u8 {
        ((self.r.sp & 0xFF00) >> 8) as u8
    }

    pub fn pc(&self) -> u16 {
        self.r.pc
    }

    pub fn pch(&self) -> u8 {
        ((self.r.pc & 0xFF00) >> 8) as u8
    }

    pub fn pcl(&self) -> u8 {
        self.r.pc as u8
    }

    pub fn wz(&self) -> u16 {
        self.r.wz
    }

    pub fn w(&self) -> u8 {
        ((self.r.wz & 0xFF00) >> 8) as u8
    }

    pub fn z(&self) -> u8 {
        (self.r.wz & 0x00FF) as u8
    }
    // }}}

    // {{{ CPU Setters
    pub fn set_r8(&mut self, r8: R8, data: u8) {
        eprintln!("Setting {:?} to {:02x}", r8, data);
        match r8 {
            R8::B => self.set_b(data),
            R8::C => self.set_c(data),
            R8::D => self.set_d(data),
            R8::E => self.set_e(data),
            R8::H => self.set_h(data),
            R8::L => self.set_l(data),
            R8::HL => todo!("Tried to set [hl]: {:04x}:{:02x}", self.hl(), data),
            R8::A => self.set_a(data),
        }
    }

    pub fn set_r16(&mut self, r16: R16, data: u16) {
        match r16 {
            R16::BC => self.set_bc(data),
            R16::DE => self.set_de(data),
            R16::HL => self.set_hl(data),
            R16::SP => self.set_sp(data),
        }
    }

    pub fn set_r16stk(&mut self, r16stk: R16stk, data: u16) {
        match r16stk {
            R16stk::BC => self.set_bc(data),
            R16stk::DE => self.set_de(data),
            R16stk::HL => self.set_hl(data),
            R16stk::AF => self.set_af(data),
        }
    }

    pub fn set_r16mem(&mut self, r16mem: R16mem, data: u16) {
        match r16mem {
            R16mem::BC => self.set_bc(data),
            R16mem::DE => self.set_de(data),
            R16mem::HLi => self.set_hl(data),
            R16mem::HLd => self.set_hl(data),
        }
    }

    pub fn set_hi(&mut self, r16: R16, data: u8) {
        match r16 {
            R16::BC => self.set_b(data),
            R16::DE => self.set_d(data),
            R16::HL => self.set_h(data),
            R16::SP => self.set_sp(self.sp() & 0x00FF | (data as u16) << 8),
        }
    }

    pub fn set_lo(&mut self, r16: R16, data: u8) {
        match r16 {
            R16::BC => self.set_c(data),
            R16::DE => self.set_e(data),
            R16::HL => self.set_l(data),
            R16::SP => self.set_sp(self.sp() & 0xFF00 | data as u16),
        }
    }

    pub fn set_m(&mut self, m: u128) {
        self.m = m
    }

    pub fn set_t(&mut self, t: u128) {
        self.t = t
    }

    pub fn set_mc(&mut self, mc: Mc) {
        self.mc = mc
    }

    pub fn set_addr(&mut self, addr: u16) {
        self.addr = addr
    }

    pub fn set_data(&mut self, data: u8) {
        self.data = data
    }

    pub fn set_ime(&mut self, ime: u8) {
        self.ime = ime
    }

    pub fn set_ir(&mut self, ir: u8) {
        self.r.ir = ir
    }

    pub fn set_ie(&mut self, ie: u8) {
        self.r.ie = ie
    }

    pub fn set_af(&mut self, af: u16) {
        self.r.af = af
    }

    pub fn set_a(&mut self, a: u8) {
        self.r.af = self.r.af & 0x00FF | (a as u16) << 8
    }

    pub fn set_zero(&mut self, z: u8) {
        if z & 0xFE != 0 {
            panic!("Invalid value used as flag z: {:02x}", z)
        }
        self.r.af = self.r.af & 0xFF7F | (z as u16) << 7
    }

    pub fn set_bcdn(&mut self, n: u8) {
        if n & 0xFE != 0 {
            panic!("Invalid value used as flag n: {:02x}", n)
        }
        self.r.af = self.r.af & 0xFFBF | (n as u16) << 6
    }

    pub fn set_bcdh(&mut self, hy: u8) {
        if hy & 0xFE != 0 {
            panic!("Invalid value used as flag hy: {:02x}", hy)
        }
        self.r.af = self.r.af & 0xFFDF | (hy as u16) << 5
    }

    pub fn set_carry(&mut self, cy: u8) {
        if cy & 0xFE != 0 {
            panic!("Invalid value used as flag cy: {:02x}", cy)
        }
        self.r.af = self.r.af & 0xFFEF | (cy as u16) << 4
    }

    pub fn set_bc(&mut self, bc: u16) {
        self.r.bc = bc
    }

    pub fn set_b(&mut self, b: u8) {
        self.r.bc = self.r.bc & 0x00FF | (b as u16) << 8
    }

    pub fn set_c(&mut self, c: u8) {
        self.r.bc = self.r.bc & 0xFF00 | (c as u16)
    }

    pub fn set_de(&mut self, de: u16) {
        self.r.de = de
    }

    pub fn set_d(&mut self, d: u8) {
        self.r.de = self.r.de & 0x00FF | (d as u16) << 8
    }

    pub fn set_e(&mut self, e: u8) {
        self.r.de = self.r.de & 0xFF00 | (e as u16)
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.r.hl = hl
    }

    pub fn set_h(&mut self, h: u8) {
        self.r.hl = self.r.hl & 0x00FF | (h as u16) << 8
    }

    pub fn set_l(&mut self, l: u8) {
        self.r.hl = self.r.hl & 0xFF00 | (l as u16)
    }

    pub fn set_sp(&mut self, sp: u16) {
        self.r.sp = sp
    }

    pub fn inc_sp(&mut self) {
        self.r.sp += 1;
    }

    pub fn dec_sp(&mut self) {
        self.r.sp -= 1;
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.r.pc = pc
    }

    pub fn set_pch(&mut self, pch: u8) {
        self.r.pc = self.r.pc & 0x00FF | (pch as u16) << 8
    }

    pub fn set_pcl(&mut self, pcl: u8) {
        self.r.pc = self.r.pc & 0xFF00 | (pcl as u16)
    }

    pub fn inc_pc(&mut self) {
        let (pc, carry) = self.pc().overflowing_add(1);
        if carry {
            panic!("PC overflowed!");
        }
        self.set_pc(pc)
    }

    pub fn set_wz(&mut self, wz: u16) {
        self.r.wz = wz
    }

    pub fn set_w(&mut self, w: u8) {
        self.r.wz = self.r.wz & 0x00FF | (w as u16) << 8
    }

    pub fn set_z(&mut self, z: u8) {
        self.r.wz = self.r.wz & 0xFF00 | (z as u16)
    }
    // }}}
}

// {{{ Defaults
#[allow(clippy::derivable_impls)]
impl std::default::Default for Cpu {
    fn default() -> Self {
        let r = Registers {
            ir: 0x00,
            ie: 0x00,
            af: 0x0000,
            bc: 0x0000,
            de: 0x0000,
            hl: 0x0000,
            sp: 0x0000,
            pc: 0x0000,
            wz: 0x0000,
        };
        Cpu {
            mem: [0u8; 0xFFFF],
            m: 0,
            t: 0,
            r,
            addr: 0x0000,
            data: 0x0000,
            ime: 0,
            cb: 0,
            mc: Mc::M1,
            executing: Cpu::nop,
            halted: false,
        }
    }
}
// }}}

// {{{ Display
impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "m: {}, t: {}, af: {:04x} bc: {:04x} de: {:04x} hl: {:04x}\na: {:02x} b: {:02x} c: {:02x} d: {:02x} e: {:02x} h: {:02x} l: {:02x}\nsp: {:04x} pc: {:04x} f: {:02x} z: {} n: {} h: {} c: {}\nir: {:02x} wz: {:04x} mc: {:?} ime: {} halted: {} cb: {}",
            self.m(),
            self.t(),
            self.af(),
            self.bc(),
            self.de(),
            self.hl(),
            self.a(),
            self.b(),
            self.c(),
            self.d(),
            self.e(),
            self.h(),
            self.l(),
            self.sp(),
            self.pc(),
            self.f(),
            self.zero(),
            self.bcdn(),
            self.bcdh(),
            self.carry(),
            self.ir(),
            self.wz(),
            self.mc(),
            self.ime(),
            self.halted,
            self.cb,
        )
    }
}
// }}}
//
//
