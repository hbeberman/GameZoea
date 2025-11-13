use crate::emu::gb::Comp;
use crate::emu::timer::*;

#[derive(Debug)]
#[allow(dead_code)]
enum Mbc {
    None,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    MMM01,
    M161,
    Huc1,
    HuC3,
}

#[allow(dead_code)]
pub struct Memory {
    owner: Comp,
    mbc: Mbc,
    mem: [u8; 0x10000],
    cartridge: Vec<u8>,
    data: u8,
    addr: u16,
    write_div: bool,
    write_tac: bool,
    tima_overflow: bool,
    cartridge_type: u8,
    rom_bank_count: u16,
    ram_bank_count: u8,
    ram_enable: bool,
    mbc1rombank: u8,
    mbc1rambank: u8,
    mbc1bankmode: u8,
}

impl Memory {
    pub fn empty() -> Self {
        Memory {
            owner: Comp::Cpu,
            mbc: Mbc::None,
            mem: [0u8; 0x10000],
            cartridge: [0u8; 0x10000].to_vec(),
            data: 0x00,
            addr: 0x0000,
            write_div: false,
            write_tac: false,
            tima_overflow: false,
            cartridge_type: 0x00,
            rom_bank_count: 0x0000,
            ram_bank_count: 0x00,
            ram_enable: false,
            mbc1rombank: 0x00,
            mbc1rambank: 0x00,
            mbc1bankmode: 0x00,
        }
    }

    pub fn new(cartridge: &[u8]) -> Self {
        let mut mem = [0u8; 0x10000];
        let (mbc, cartridge_type) = Memory::mbc_decode(cartridge);
        let rom_bank_count = Memory::rom_bank_count_decode(cartridge);
        let ram_bank_count = Memory::ram_bank_count_decode(cartridge);
        mem[0x0000..cartridge.len()].copy_from_slice(cartridge);
        mem[0xFF00] = 0xFF; // Stub Joypad
        let mem = Memory {
            owner: Comp::Cpu,
            mbc,
            mem,
            cartridge: cartridge.to_vec(),
            data: 0x00,
            addr: 0x0000,
            write_div: false,
            write_tac: false,
            tima_overflow: false,
            cartridge_type,
            rom_bank_count,
            ram_bank_count,
            ram_enable: false,
            mbc1rombank: 0x00,
            mbc1rambank: 0x00,
            mbc1bankmode: 0x00,
        };
        eprintln!(
            "MEM: rom_bank_count:#{} ram_bank_count:#{} mbc:{:?}",
            mem.rom_bank_count, mem.ram_bank_count, mem.mbc
        );
        mem
    }

    fn mbc_decode(cartridge: &[u8]) -> (Mbc, u8) {
        let cartridge_type = cartridge[0x147];
        let mbc = match cartridge_type {
            0x00 => Mbc::None,
            0x01..=0x03 => Mbc::MBC1,
            x => todo!("MBC {:02X} not implemented!", x),
        };
        eprintln!("MBC {:?} found", mbc);
        (mbc, cartridge_type)
    }

    fn rom_bank_count_decode(cartridge: &[u8]) -> u16 {
        let val = cartridge[0x0148];
        match val {
            0x00..=0x08 => 0b1 << (val + 1),
            x => panic!("Invalid rom size value:{:02X}", x),
        }
    }

    fn ram_bank_count_decode(cartridge: &[u8]) -> u8 {
        let val = cartridge[0x0149];
        match val {
            0x00 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            x => panic!("Invalid ram size value:{:02X}", x),
        }
    }

    pub fn read(&mut self) {
        self.data = match self.addr {
            0x0000..=0x7FFF => self.mbc_read(),
            0xA000..=0xBFFF => self.mbc_read(),
            _ => self.mem[self.addr as usize],
        }
    }

    pub fn dbg_read_16(&self, addr: u16) -> [u8; 16] {
        let start = addr as usize;
        self.mem[start..start + 16]
            .try_into()
            .expect("dbg_read_16 used out of bounds")
    }

    pub fn dbg_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write(&mut self) {
        let addr = self.addr();
        let data = self.data();

        if self.owner == Comp::Cpu && self.tima_overflow && addr == TIMA {
            return;
        }
        match addr {
            // 0x8000 => panic!("MEM WRITE!!!! data:{:02X}", data),
            0x0000..0x8000 => self.mbc_rom_write(),
            0x8000..0xA000 => self.mem[addr as usize] = data, // 8 KiB VRAM (GBC Bank 00-01)
            0xA000..0xC000 => self.mem[addr as usize] = data, // 8 KiB External RAM
            0xC000..0xD000 => self.mem[addr as usize] = data, // 4 KiB Work RAM
            0xD000..0xE000 => self.mem[addr as usize] = data, // 4 KiB Work RAM (GBC Bank 01-07)
            0xE000..0xFE00 => self.mem[(addr & 0x3FFF) as usize] = data, // Echo Ram
            0xFE00..0xFEA0 => self.write_oam(addr, data),
            0xFEA0..0xFF00 => panic!("Memory write to not usable: {:04x}:{:02x}", addr, data),
            0xFF04 => {
                self.mem[addr as usize] = 0;
                self.write_div = true;
            }
            0xFF07 => self.set_tac(data),
            0xFF00..0xFF80 => self.mem[addr as usize] = data, // I/O Registers
            0xFF80..0xFFFF => self.mem[addr as usize] = data, // High RAM (HRAM)
            0xFFFF => self.mem[addr as usize] = data,         // Interrupt Enable
        }
    }

    pub fn dbg_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data
    }

    pub fn bulk_write(&mut self, addr: u16, newmem: &[u8]) {
        self.mem[addr as usize..newmem.len()].copy_from_slice(newmem);
    }

    pub fn addr(&self) -> u16 {
        self.addr
    }

    pub fn data(&self) -> u8 {
        self.data
    }

    pub fn set_addr(&mut self, addr: u16) {
        self.addr = addr
    }

    pub fn set_data(&mut self, data: u8) {
        self.data = data
    }

    pub fn set_tac(&mut self, tac: u8) {
        self.mem[0xFF07] = (self.mem[0xFF07] & 0xF8) + (tac & 0x07);
        self.write_tac = true;
    }

    pub fn set_tima_overflow(&mut self, tima_overflow: bool) {
        self.tima_overflow = tima_overflow;
    }

    pub fn tima_overflow(&self) -> bool {
        self.tima_overflow
    }

    pub fn owner(&self) -> Comp {
        self.owner.clone()
    }

    pub fn set_owner(&mut self, owner: Comp) {
        self.owner = owner;
    }

    pub fn check_write_div(&mut self) -> bool {
        let result = self.write_div;
        self.write_div = false;
        result
    }

    pub fn check_write_tac(&mut self) -> bool {
        let result = self.write_tac;
        self.write_tac = false;
        result
    }

    pub fn write_oam(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data // 4 KiB Work RAM (GBC Bank 01-07)
    }

    pub fn mbc_rom_write(&mut self) {
        match &self.mbc {
            Mbc::None => panic!("Attempted to write to rom on MBC None"),
            Mbc::MBC1 => self.mbc1_register_write(),
            x => todo!(
                "ROM write on unimplemented MBC:{:?} addr:{:04X}",
                x,
                self.addr
            ),
        }
    }

    pub fn mbc1_register_write(&mut self) {
        match self.addr {
            0x0000..=0x1FFF => self.ram_enable = self.data & 0x0A == 0x0A,
            0x2000..=0x3FFF => self.mbc1rombank = self.data & 0x1F,
            0x4000..=0x5FFF => self.mbc1rambank = self.data & 0x3,
            0x6000..=0x7FFF => self.mbc1bankmode = self.data & 0x1,
            _ => unreachable!("Invalid addr:{:04X} for MBC1 write", self.addr),
        }
    }

    pub fn mbc_read(&mut self) -> u8 {
        match &self.mbc {
            Mbc::None => self.mem[self.addr as usize],
            Mbc::MBC1 => self.mbc1_read(),
            x => todo!("Read on unimplemented MBC:{:?} addr:{:04X}", x, self.addr),
        }
    }

    pub fn mbc1_read(&mut self) -> u8 {
        match self.addr {
            0x0000..=0x7FFF => {
                let cart_addr = self.mbc1_rom_addr(self.addr);
                self.cartridge[cart_addr]
            }
            0xA000..=0xBFFF => self.mem[self.addr as usize],
            _ => unreachable!("Invalid mbc1 read decode addr:{:04X}", self.addr),
        }
    }

    fn mbc1_rom_addr(&self, addr: u16) -> usize {
        let offset = (addr as usize) & 0x3FFF;
        let bank = match addr {
            0x0000..=0x3FFF => self.mbc1_fixed_rom_bank(),
            0x4000..=0x7FFF => self.mbc1_switchable_rom_bank(),
            _ => unreachable!("Invalid ROM decode addr:{:04X}", addr),
        };
        let rom_len = self.cartridge.len();
        debug_assert!(rom_len > 0, "cartridge must contain data");
        ((bank << 14) | offset) % rom_len
    }

    fn mbc1_fixed_rom_bank(&self) -> usize {
        if self.mbc1bankmode & 0x1 == 0x1 {
            let bank = ((self.mbc1rambank as usize) & 0x3) << 5;
            self.mbc1_normalize_bank(bank, false)
        } else {
            0
        }
    }

    fn mbc1_switchable_rom_bank(&self) -> usize {
        let upper = if self.mbc1bankmode & 0x1 == 0x0 {
            ((self.mbc1rambank as usize) & 0x3) << 5
        } else {
            0
        };
        let mut bank = upper | ((self.mbc1rombank as usize) & 0x1F);
        if (bank & 0x1F) == 0 {
            bank += 1;
        }
        self.mbc1_normalize_bank(bank, true)
    }

    fn mbc1_normalize_bank(&self, bank: usize, require_non_zero: bool) -> usize {
        let total = self.mbc1_total_rom_banks();
        if total == 0 {
            return 0;
        }
        let mut bank = bank % total;
        if require_non_zero && total > 1 && bank == 0 {
            bank = 1;
        }
        bank
    }

    fn mbc1_total_rom_banks(&self) -> usize {
        let banks = self.cartridge.len() / 0x4000;
        if banks == 0 { 1 } else { banks }
    }
}
