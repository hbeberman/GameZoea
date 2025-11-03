pub struct Memory {
    mem: [u8; 0x10000],
    data: u8,
    addr: u16,
}

impl Memory {
    pub fn empty() -> Self {
        Memory {
            mem: [0u8; 0x10000],
            data: 0x00,
            addr: 0x0000,
        }
    }

    pub fn new(cartridge: &[u8]) -> Self {
        let mut mem = [0u8; 0x10000];
        mem[0x0000..cartridge.len()].copy_from_slice(cartridge);
        Memory {
            mem,
            data: 0x00,
            addr: 0x0000,
        }
    }

    pub fn read(&mut self) {
        self.data = self.mem[self.addr as usize];
    }

    pub fn dbg_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn write(&mut self) {
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
            0xFF04 => self.mem[0xFF04] = 0x00, // Writes to DIV register set it to 0x00
            0xFF00..0xFF80 => self.mem[addr as usize] = data, // I/O Registers
            0xFF80..0xFFFF => self.mem[addr as usize] = data, // High RAM (HRAM)
            0xFFFF => self.mem[addr as usize] = data, // Interrupt Enable
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
}
