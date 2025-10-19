use std::fmt;

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
    mem: [u16; 0xFFFF],
    m: u128,
    t: u128,
    r: Registers,
    addr: u16,
    data: u8,
}

impl Registers {}

impl Cpu {
    // Cycle Functions
    pub fn tick(&mut self) {
        self.t += 1;
        if self.t.is_multiple_of(4) {
            self.m += 1;
        }
    }

    // Memory Functions
    pub fn mem_read(&self, addr: u16) -> u16 {
        self.mem[addr as usize]
    }

    pub fn mem_write(&mut self, addr: u16, data: u16) {
        match addr {
            0x0000..0x4000 => todo!("Memory write to ROM bank 00: {:04x}:{:04x}", addr, data),
            0x4000..0x8000 => todo!("Memory write to ROM bank 01-NN: {:04x}:{:04x}", addr, data),
            0x8000..0xA000 => self.mem[addr as usize] = data, // 8 KiB VRAM (GBC Bank 00-01)
            0xA000..0xC000 => self.mem[addr as usize] = data, // 8 KiB External RAM
            0xC000..0xD000 => self.mem[addr as usize] = data, // 4 KiB Work RAM
            0xD000..0xE000 => self.mem[addr as usize] = data, // 4 KiB Work RAM (GBC Bank 01-07)
            0xE000..0xFE00 => panic!("Memory write to echo RAM: {:04x}:{:04x}", addr, data),
            0xFE00..0xFEA0 => todo!("Memory write to OAM: {:04x}:{:04x}", addr, data),
            0xFEA0..0xFF00 => panic!("Memory write to not usable: {:04x}:{:04x}", addr, data),
            0xFF00..0xFF80 => todo!("Memory write to I/O registers: {:04x}:{:04x}", addr, data),
            0xFF80..0xFFFF => todo!("Memory write to HRAM: {:04x}:{:04x}", addr, data),
            0xFFFF => todo!("Memory write to IE register: {:04x}:{:04x}", addr, data),
        }
    }

    // Register Getters
    pub fn m(&self) -> u128 {
        self.m
    }

    pub fn t(&self) -> u128 {
        self.t
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

    pub fn n(&self) -> u8 {
        ((self.r.af & 0x40) >> 6) as u8
    }

    pub fn hy(&self) -> u8 {
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

    // Register Setters
    pub fn set_m(&mut self, m: u128) {
        self.m = m
    }

    pub fn set_t(&mut self, t: u128) {
        self.t = t
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
        self.r.bc = self.r.bc & 0x00FF | (d as u16) << 8
    }

    pub fn set_e(&mut self, e: u8) {
        self.r.bc = self.r.bc & 0xFF00 | (e as u16)
    }

    pub fn set_hl(&mut self, hl: u16) {
        self.r.hl = hl
    }

    pub fn set_h(&mut self, h: u8) {
        self.r.bc = self.r.bc & 0x00FF | (h as u16) << 8
    }

    pub fn set_l(&mut self, l: u8) {
        self.r.bc = self.r.bc & 0xFF00 | (l as u16)
    }

    pub fn set_sp(&mut self, sp: u16) {
        self.r.sp = sp
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.r.pc = pc
    }
}

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
            mem: [0u16; 0xFFFF],
            m: 0,
            t: 0,
            r,
            addr: 0x0000,
            data: 0x0000,
        }
    }
}

impl std::fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "m: {}, t: {}, af: {:04x} bc: {:04x} de: {:04x} hl: {:04x}\na: {:02x} b: {:02x} c: {:02x} d: {:02x} e: {:02x} h: {:02x} l: {:02x}\nsp: {:04x} pc: {:04x} f: {:02x} z: {} n: {} h: {} c: {}",
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
            self.n(),
            self.hy(),
            self.cy(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Register Tests
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

    // Cycle Tests
    #[test]
    fn cpu_t_tick() {
        let mut cpu = Cpu::default();
        assert_eq!(cpu.t(), 0);
        for i in 1..1000 {
            cpu.tick();
            assert_eq!(cpu.t(), i);
            assert_eq!(cpu.m(), i / 4);
        }
    }

    // Memory Tests
    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to ROM bank 00: 0000:abcd")]
    fn mem_rom_write() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x0000, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to ROM bank 01-NN: 4000:abcd")]
    fn mem_rom_bankable_write() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x4000, 0xABCD);
    }

    #[test]
    fn mem_write_read_vram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x8000, 0xABCD);
        assert_eq!(cpu.mem_read(0x8000), 0xABCD);
    }

    #[test]
    fn mem_write_read_external_ram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xA000, 0xABCD);
        assert_eq!(cpu.mem_read(0xA000), 0xABCD);
    }

    #[test]
    fn mem_write_read_work_ram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xC000, 0xABCD);
        assert_eq!(cpu.mem_read(0xC000), 0xABCD);
    }

    #[test]
    fn mem_write_read_work_ram_bankable() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xD000, 0xABCD);
        assert_eq!(cpu.mem_read(0xD000), 0xABCD);
    }

    #[test]
    #[should_panic(expected = "Memory write to echo RAM: e000:abcd")]
    fn mem_write_echo() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xE000, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to OAM: fe00:abcd")]
    fn mem_write_oam() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFE00, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "Memory write to not usable: fea0:abcd")]
    fn mem_write_not_usable() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFEA0, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to I/O registers: ff00:abcd")]
    fn mem_write_io() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFF00, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to HRAM: ff80:abcd")]
    fn mem_write_hram() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFF80, 0xABCD);
    }

    #[test]
    #[should_panic(expected = "not yet implemented: Memory write to IE register: ffff:abcd")]
    fn mem_write_ie() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0xFFFF, 0xABCD);
    }
}
