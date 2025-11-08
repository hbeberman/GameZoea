pub struct Memory {
    mem: [u8; 0x10000],
    data: u8,
    addr: u16,
    write_div: bool,
    write_tac: bool,
}

impl Memory {
    pub fn empty() -> Self {
        Memory {
            mem: [0u8; 0x10000],
            data: 0x00,
            addr: 0x0000,
            write_div: false,
            write_tac: false,
        }
    }

    pub fn new(cartridge: &[u8]) -> Self {
        let mut mem = [0u8; 0x10000];
        mem[0x0000..cartridge.len()].copy_from_slice(cartridge);
        mem[0xFF05] = 0x00; // TIMA initial value after DMG boot ROM
        mem[0xFF07] = 0xF8; // TAC initial value after DMG boot ROM
        Memory {
            mem,
            data: 0x00,
            addr: 0x0000,
            write_div: false,
            write_tac: false,
        }
    }

    pub fn read(&mut self) {
        self.data = self.mem[self.addr as usize];
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
        match addr {
            //            0x8000 => panic!("MEM WRITE!!!! data:{:02X}", data),
            0x0000..0x4000 => todo!("Memory write to ROM bank 00: {:04x}:{:02x}", addr, data),
            0x4000..0x8000 => todo!("Memory write to ROM bank 01-NN: {:04x}:{:02x}", addr, data),
            0x8000..0xA000 => self.mem[addr as usize] = data, // 8 KiB VRAM (GBC Bank 00-01)
            0xA000..0xC000 => self.mem[addr as usize] = data, // 8 KiB External RAM
            0xC000..0xD000 => self.mem[addr as usize] = data, // 4 KiB Work RAM
            0xD000..0xE000 => self.mem[addr as usize] = data, // 4 KiB Work RAM (GBC Bank 01-07)
            0xE000..0xFE00 => panic!("Memory write to echo RAM: {:04x}:{:02x}", addr, data),
            0xFE00..0xFEA0 => todo!("Memory write to OAM: {:04x}:{:02x}", addr, data),
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
}
