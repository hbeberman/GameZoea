use std::fmt;

#[allow(dead_code)]
const M54: u8 = 0b00110000;
const M543: u8 = 0b00111000;

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
}

pub struct Cpu {
    mem: [u8; 0xFFFF],
    m: u128,
    t: u128,
    r: Registers,
    addr: u16,
    data: u8,
    mc: Mc,
    executing: fn(&mut Cpu),
}

impl Registers {}

impl Cpu {
    // {{{ Execute Functions
    pub fn decode(&mut self) -> fn(&mut Cpu) {
        match self.ir() {
            0x00 => Cpu::noop, // noop
            //
            0x04 | 0x24 | 0x14 | 0x0C | 0x2C | 0x1C | 0x3C => Cpu::inc_r8,
            //
            _ => panic!("Opcode not implemented: 0x{:02x}", self.ir()),
        }
    }

    pub fn execute(&mut self) {
        eprintln!("Execute called!\n{}", self);
        (self.executing)(self);
        self.mc = self.mc.next();
    }

    pub fn fetch_next(&mut self) {
        let (pc, overflow) = self.pc().overflowing_add(1);
        if overflow {
            panic!("PC overflowed!")
        }
        self.set_ir(self.mem_read(self.pc()));
        self.mc = M0;
        self.executing = self.decode();
        (self.executing)(self);
        self.set_pc(pc);
    }

    pub fn noop(&mut self) {
        match self.mc {
            Mc::M1 => self.fetch_next(),
            Mc::M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in noop: {:?}", self.mc),
        }
    }

    pub fn inc_r8(&mut self) {
        match self.mc {
            M1 => {
                let r8 = R8::from((self.ir() & M543) >> 3);
                eprintln!("inc_r8 self.ir:{:02x}", self.ir());
                eprintln!("inc_r8 self.ir mask:{:02x}", self.ir() & M543);
                eprintln!("inc_r8 self.ir mask shift:{:02x}", self.ir() & M543 >> 3);
                eprintln!("inc_r8 r8:{:?}", r8);
                eprintln!("{}", self);

                let (result, overflow) = self.r8(r8).overflowing_add(1);

                eprintln!("Incrementing {:?} to {:02x}", r8, result);

                if result == 0 {
                    self.set_z(1)
                } else {
                    self.set_z(0)
                }

                self.set_bcdn(0);

                if overflow {
                    self.set_cy(1)
                } else {
                    self.set_cy(0)
                }

                self.set_r8(r8, result);
                self.fetch_next()
            }
            M0 => self.set_mc(M2),
            _ => panic!("Invalid mc in inc_r8: {:?}", self.mc),
        }
    }

    // }}}

    // {{{ Cycle Functions
    pub fn tick(&mut self) {
        self.t += 1;
        if self.t.is_multiple_of(4) {
            self.m += 1;
            self.execute()
        }
    }

    pub fn tick4(&mut self) {
        for _ in 0..4 {
            self.tick()
        }
    }
    // }}}

    // {{{ Memory Functions
    pub fn mem_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
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
            0xFF80..0xFFFF => todo!("Memory write to HRAM: {:04x}:{:02x}", addr, data),
            0xFFFF => todo!("Memory write to IE register: {:04x}:{:02x}", addr, data),
        }
    }

    pub fn mem_dbg_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data
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
                self.mem_read(self.hl())
            ),
            R8::A => self.a(),
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

    pub fn z(&self) -> u8 {
        ((self.r.af & 0x80) >> 7) as u8
    }

    pub fn bcdn(&self) -> u8 {
        ((self.r.af & 0x40) >> 6) as u8
    }

    pub fn bcdh(&self) -> u8 {
        ((self.r.af & 0x20) >> 5) as u8
    }

    pub fn cy(&self) -> u8 {
        ((self.r.af & 0x10) >> 4) as u8
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

    pub fn pc(&self) -> u16 {
        self.r.pc
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

    pub fn set_z(&mut self, z: u8) {
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

    pub fn set_cy(&mut self, cy: u8) {
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

    pub fn set_pc(&mut self, pc: u16) {
        self.r.pc = pc
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
        };
        Cpu {
            mem: [0u8; 0xFFFF],
            m: 0,
            t: 0,
            r,
            addr: 0x0000,
            data: 0x0000,
            mc: Mc::M1,
            executing: Cpu::noop,
        }
    }
}
// }}}

// {{{ Display
impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "m: {}, t: {}, af: {:04x} bc: {:04x} de: {:04x} hl: {:04x}\na: {:02x} b: {:02x} c: {:02x} d: {:02x} e: {:02x} h: {:02x} l: {:02x}\nsp: {:04x} pc: {:04x} f: {:02x} z: {} n: {} h: {} c: {}\nir: {:02x} mc: {:?}",
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
            self.z(),
            self.bcdn(),
            self.bcdh(),
            self.cy(),
            self.ir(),
            self.mc(),
        )
    }
}
// }}}

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
        cpu.set_z(1);
        cpu.set_bcdn(1);
        cpu.set_bcdh(1);
        cpu.set_cy(1);
        assert_eq!(cpu.f(), 0xF0);
    }

    #[test]
    fn cpu_flag_gets() {
        let mut cpu = Cpu::default();
        cpu.set_af(0x00F0);
        assert_eq!(cpu.z(), 1);
        assert_eq!(cpu.bcdn(), 1);
        assert_eq!(cpu.bcdh(), 1);
        assert_eq!(cpu.cy(), 1);
    }

    #[test]
    #[should_panic(expected = "Invalid value used as flag z: 02")]
    fn cpu_flag_z_invalid() {
        let mut cpu = Cpu::default();
        cpu.set_z(2);
        assert_eq!(cpu.z(), 1);
    }
    // }}}

    // {{{ Cycle Tests
    #[test]
    fn cpu_t_tick() {
        let mut cpu = Cpu::default();
        assert_eq!(cpu.t(), 0);
        for i in 1..16 {
            cpu.tick();
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
        cpu.mem_write(0x0000, 0xAB);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to ROM bank 01-NN: 4000:ab")]
    fn mem_rom_bankable_write() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x4000, 0xAB);
    }

    #[test]
    fn mem_write_read_vram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x8000, 0xAB);
        assert_eq!(cpu.mem_read(0x8000), 0xAB);
    }

    #[test]
    fn mem_write_read_external_ram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xA000, 0xAB);
        assert_eq!(cpu.mem_read(0xA000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xC000, 0xAB);
        assert_eq!(cpu.mem_read(0xC000), 0xAB);
    }

    #[test]
    fn mem_write_read_work_ram_bankable() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xD000, 0xAB);
        assert_eq!(cpu.mem_read(0xD000), 0xAB);
    }

    #[test]
    #[should_panic(expected = "Memory write to echo RAM: e000:ab")]
    fn mem_write_echo() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xE000, 0xAB);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to OAM: fe00:ab")]
    fn mem_write_oam() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFE00, 0xAB);
    }

    #[test]
    #[should_panic(expected = "Memory write to not usable: fea0:ab")]
    fn mem_write_not_usable() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFEA0, 0xAB);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to I/O registers: ff00:ab")]
    fn mem_write_io() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFF00, 0xAB);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to HRAM: ff80:ab")]
    fn mem_write_hram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFF80, 0xAB);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to IE register: ffff:ab")]
    fn mem_write_ie() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFFFF, 0xAB);
    }
    // }}}

    // {{{ Execute Tests
    #[test]
    fn execute_noop() {
        let mut cpu = Cpu::default();
        cpu.tick4();
        assert_eq!(cpu.pc(), 0x0001);
        cpu.tick4();
        cpu.tick4();
        assert_eq!(cpu.pc(), 0x0003);
    }

    #[test]
    fn execute_inc_r8() {
        let mut cpu = Cpu::default();
        let mem = [0x04, 0x04, 0x04, 0x24, 0x14, 0x0C, 0x2C, 0x1C, 0x3C];
        cpu.mem[..mem.len()].copy_from_slice(&mem);
        cpu.set_b(0xFF);
        cpu.tick4();
        cpu.tick4();
        assert_eq!(cpu.b(), 0);
        assert_eq!(cpu.cy(), 1);
        assert_eq!(cpu.z(), 1);
        cpu.tick4();
        assert_eq!(cpu.cy(), 0);
        assert_eq!(cpu.z(), 0);

        for _ in 0..10 {
            cpu.tick4();
        }
        assert_eq!(cpu.b(), 2);
        assert_eq!(cpu.c(), 1);
        assert_eq!(cpu.d(), 1);
        assert_eq!(cpu.e(), 1);
        assert_eq!(cpu.h(), 1);
        assert_eq!(cpu.l(), 1);
        assert_eq!(cpu.a(), 1);
    }
    // }}}
}
