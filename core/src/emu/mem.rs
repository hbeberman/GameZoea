use crate::emu::gb::Comp;
use crate::emu::regs::*;
use std::time::{Duration, Instant};

const DMA_TRANSFER_CYCLES: usize = 160 * 4;
const DMA_START_DELAY_CYCLES: u8 = 8;
const OAM_START: usize = 0xFE00;
const OAM_LEN: usize = 0xA0;

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
    dma: usize,
    dma_start_delay: u8,
    dma_delay_block: bool,
    dma_source: Option<usize>,
    oam_busy: bool,
    vram_busy: bool,
    mbc: Mbc,
    mem: [u8; 0x10000],
    cartridge: Vec<u8>,
    cart_ram: Vec<u8>,
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
    mbc3_rom_bank: u8,
    mbc3_ram_bank: u8,
    mbc3_rtc_select: Option<u8>,
    rtc_seconds: u8,
    rtc_minutes: u8,
    rtc_hours: u8,
    rtc_day_counter: u16,
    rtc_day_carry: bool,
    rtc_halt: bool,
    rtc_last_update: Instant,
    rtc_latched: [u8; 5],
    rtc_latch_active: bool,
    rtc_latch_prev: u8,
}

impl Memory {
    pub fn empty() -> Self {
        Memory {
            owner: Comp::Cpu,
            dma: 0,
            dma_start_delay: 0,
            dma_delay_block: false,
            dma_source: None,
            oam_busy: false,
            vram_busy: false,
            mbc: Mbc::None,
            mem: [0u8; 0x10000],
            cartridge: [0u8; 0x10000].to_vec(),
            cart_ram: Vec::new(),
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
            mbc3_rom_bank: 0x01,
            mbc3_ram_bank: 0x00,
            mbc3_rtc_select: None,
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_day_counter: 0,
            rtc_day_carry: false,
            rtc_halt: false,
            rtc_last_update: Instant::now(),
            rtc_latched: [0; 5],
            rtc_latch_active: false,
            rtc_latch_prev: 0,
        }
    }

    pub fn new(cartridge: &[u8]) -> Self {
        let mut mem = [0u8; 0x10000];
        let (mbc, cartridge_type) = Memory::mbc_decode(cartridge);
        let rom_bank_count = Memory::rom_bank_count_decode(cartridge);
        let ram_bank_count = Memory::ram_bank_count_decode(cartridge);
        let cart_ram_len = (ram_bank_count as usize).saturating_mul(0x2000);
        let cart_ram = vec![0; cart_ram_len];
        eprintln!(
            "MBC {:?} found. cartridge_type:{:02X} rom_banks:#{} ram_banks:#{}",
            mbc, cartridge_type, rom_bank_count, ram_bank_count
        );
        let cap = if cartridge.len() >= 0x8000 {
            0x8000
        } else {
            cartridge.len()
        };
        mem[0x0000..cap].copy_from_slice(&cartridge[0x0000..cap]);
        let mem = Memory {
            owner: Comp::Cpu,
            dma: 0,
            dma_start_delay: 0,
            dma_delay_block: false,
            dma_source: None,
            oam_busy: false,
            vram_busy: false,
            mbc,
            mem,
            cartridge: cartridge.to_vec(),
            cart_ram,
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
            mbc3_rom_bank: 0x01,
            mbc3_ram_bank: 0x00,
            mbc3_rtc_select: None,
            rtc_seconds: 0,
            rtc_minutes: 0,
            rtc_hours: 0,
            rtc_day_counter: 0,
            rtc_day_carry: false,
            rtc_halt: false,
            rtc_last_update: Instant::now(),
            rtc_latched: [0; 5],
            rtc_latch_active: false,
            rtc_latch_prev: 0,
        };
        eprintln!(
            "MEM: rom_bank_count:#{} ram_bank_count:#{} mbc:{:?}",
            mem.rom_bank_count, mem.ram_bank_count, mem.mbc
        );
        mem
    }

    fn mbc_decode(cartridge: &[u8]) -> (Mbc, u8) {
        let cartridge_type = cartridge[CART_TYPE];
        let mbc = match cartridge_type {
            0x00 => Mbc::None,
            0x01..=0x03 => Mbc::MBC1,
            0x0F..=0x13 => Mbc::MBC3,
            x => todo!("MBC {:02X} not implemented!", x),
        };
        (mbc, cartridge_type)
    }

    fn rom_bank_count_decode(cartridge: &[u8]) -> u16 {
        let val = cartridge[CART_SIZE];
        match val {
            0x00..=0x08 => 0b1 << (val + 1),
            0x52 => 72,
            0x53 => 80,
            0x54 => 96,
            x => panic!("Invalid rom size value:{:02X}", x),
        }
    }

    fn ram_bank_count_decode(cartridge: &[u8]) -> u8 {
        let val = cartridge[CART_RAM];
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
        let addr = self.addr;
        if self.owner == Comp::Cpu {
            if self.dma_blocks_cpu(addr) {
                self.data = 0xFF;
                return;
            }
            if self.oam_busy && (0xFE00..0xFEA0).contains(&addr) {
                self.data = 0xFF;
                return;
            }
            if self.vram_busy && (0x8000..0xA000).contains(&addr) {
                self.data = 0xFF;
                return;
            }
        }
        if self.owner == Comp::Ppu && self.dma_blocks_oam(addr) {
            self.data = 0xFF;
            return;
        }
        self.data = self.read_mapped(addr);
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

        if self.owner == Comp::Cpu {
            if self.tima_overflow && addr == TIMA {
                return;
            }
            if addr == DMA {
                let start = (data as usize) << 8;
                let was_blocking = self.dma_bus_blocked();
                self.dma_start_delay = DMA_START_DELAY_CYCLES;
                self.dma_delay_block = was_blocking;
                self.dma_source = Some(start);
                self.dma = DMA_TRANSFER_CYCLES;
            }
            if self.dma_blocks_cpu(addr) {
                return;
            }
            if self.oam_busy && (0xFE00..0xFEA0).contains(&addr) {
                return;
            }
            if self.vram_busy && (0x8000..0xA000).contains(&addr) {
                return;
            }
        }
        if self.owner == Comp::Ppu && self.dma_blocks_oam(addr) {
            return;
        }

        match addr {
            0x0000..0x8000 => self.mbc_rom_write(),
            0x8000..0xA000 => self.mem[addr as usize] = data, // 8 KiB VRAM (GBC Bank 00-01)
            0xA000..0xC000 => {
                if !self.mbc_ram_write(addr, data) {
                    self.mem[addr as usize] = data; // 8 KiB External RAM (no mapper handling)
                }
            }
            0xC000..0xD000 => {
                self.mem[addr as usize] = data; // 4 KiB Work RAM
                let echo = addr + 0x2000;
                if echo < 0xFE00 {
                    self.mem[echo as usize] = data;
                }
            }
            0xD000..0xE000 => {
                self.mem[addr as usize] = data; // 4 KiB Work RAM (GBC Bank 01-07)
                if addr < 0xDE00 {
                    let echo = addr + 0x2000;
                    if echo < 0xFE00 {
                        self.mem[echo as usize] = data;
                    }
                }
            }
            0xE000..0xFE00 => {
                let base = (addr - 0x2000) as usize;
                self.mem[base] = data; // Echo Ram mirrors C000-DDFF
                self.mem[addr as usize] = data;
            }
            0xFE00..0xFEA0 => self.write_oam(addr, data),
            0xFEA0..0xFF00 => (), // Not Usable
            P1 => {
                if self.owner == Comp::Cpu {
                    let current = self.mem[addr as usize] & 0x0F;
                    let select = data & 0x30;
                    self.mem[addr as usize] = 0xC0 | select | current;
                } else {
                    self.mem[addr as usize] = data;
                }
            }
            DIV => {
                self.mem[addr as usize] = 0;
                self.write_div = true;
            }
            TAC => self.set_tac(data),
            0xFF01..0xFF80 => self.mem[addr as usize] = data, // I/O Registers
            0xFF80..0xFFFF => self.mem[addr as usize] = data, // High RAM (HRAM)
            0xFFFF => self.mem[addr as usize] = data,         // Interrupt Enable
        }
    }

    pub fn dbg_write(&mut self, addr: u16, data: u8) {
        if (0xA000..=0xBFFF).contains(&addr) {
            match self.mbc {
                Mbc::MBC1 => {
                    self.write_cart_ram_bank(self.mbc1_active_ram_bank(), addr, data);
                    return;
                }
                Mbc::MBC3 => {
                    if self.mbc3_rtc_select.is_none() {
                        self.write_cart_ram_bank(self.mbc3_active_ram_bank(), addr, data);
                        return;
                    }
                }
                _ => {}
            }
        }

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

    pub fn read_vram(&mut self, addr: u16) -> u8 {
        match self.owner {
            Comp::Cpu => {
                if self.vram_busy {
                    return 0xFF;
                }
            }
            Comp::Ppu => {
                if self.dma_bus_blocked() {
                    return 0xFF;
                }
            }
            _ => (),
        };
        self.mem[addr as usize]
    }

    pub fn write_oam(&mut self, addr: u16, data: u8) {
        if self.owner == Comp::Cpu && (self.oam_busy || self.vram_busy) {
            return;
        }
        self.mem[addr as usize] = data
    }

    pub fn tick(&mut self, _t: u128) {
        if self.dma_start_delay > 0 {
            self.dma_start_delay -= 1;
            if self.dma_start_delay == 0 {
                if let Some(start) = self.dma_source.take() {
                    self.copy_oam_from(start);
                }
                self.dma_delay_block = false;
            }
            return;
        }

        if self.dma != 0 {
            self.dma -= 1;
        }
    }

    pub fn set_oam_busy(&mut self, oam_busy: bool) {
        self.oam_busy = oam_busy;
    }

    pub fn set_vram_busy(&mut self, vram_busy: bool) {
        self.vram_busy = vram_busy;
    }

    fn dma_blocks_cpu(&self, addr: u16) -> bool {
        addr < 0xFF00 && self.dma_bus_blocked()
    }

    fn dma_blocks_oam(&self, addr: u16) -> bool {
        (0xFE00..0xFEA0).contains(&addr) && self.dma_bus_blocked()
    }

    fn dma_bus_blocked(&self) -> bool {
        if self.dma_start_delay > 0 {
            self.dma_delay_block
        } else {
            self.dma != 0
        }
    }

    fn copy_oam_from(&mut self, start: usize) {
        let base = (start & 0xFFFF) as u16;
        for offset in 0..OAM_LEN {
            let addr = base.wrapping_add(offset as u16);
            let value = self.read_mapped(addr);
            self.mem[OAM_START + offset] = value;
        }
    }

    pub fn mbc_rom_write(&mut self) {
        match &self.mbc {
            Mbc::None => (),
            Mbc::MBC1 => self.mbc1_register_write(),
            Mbc::MBC3 => self.mbc3_register_write(),
            x => todo!(
                "ROM write on unimplemented MBC:{:?} addr:{:04X}",
                x,
                self.addr
            ),
        }
    }

    fn mbc_ram_write(&mut self, addr: u16, data: u8) -> bool {
        match self.mbc {
            Mbc::MBC1 => {
                self.mbc1_ram_write(addr, data);
                true
            }
            Mbc::MBC3 => {
                self.mbc3_ram_or_rtc_write(addr, data);
                true
            }
            _ => false,
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

    pub fn mbc3_register_write(&mut self) {
        match self.addr {
            0x0000..=0x1FFF => self.ram_enable = self.data & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let mut bank = self.data & 0x7F;
                if bank == 0 {
                    bank = 1;
                }
                self.mbc3_rom_bank = bank;
            }
            0x4000..=0x5FFF => {
                if self.data <= 0x03 {
                    self.mbc3_ram_bank = self.data & 0x03;
                    self.mbc3_rtc_select = None;
                } else if (0x08..=0x0C).contains(&self.data) {
                    self.mbc3_rtc_select = Some(self.data);
                }
            }
            0x6000..=0x7FFF => self.latch_rtc_data(self.data & 0x01),
            _ => unreachable!("Invalid addr:{:04X} for MBC3 write", self.addr),
        }
    }

    fn read_mapped(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.mbc_read(addr),
            _ => self.mem[addr as usize],
        }
    }

    pub fn mbc_read(&mut self, addr: u16) -> u8 {
        match &self.mbc {
            Mbc::None => self.mem[addr as usize],
            Mbc::MBC1 => self.mbc1_read(addr),
            Mbc::MBC3 => self.mbc3_read(addr),
            x => todo!("Read on unimplemented MBC:{:?} addr:{:04X}", x, addr),
        }
    }

    pub fn mbc1_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                let cart_addr = self.mbc1_rom_addr(addr);
                self.cartridge[cart_addr]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enable {
                    return 0xFF;
                }
                self.read_cart_ram_bank(self.mbc1_active_ram_bank(), addr)
            }
            _ => unreachable!("Invalid mbc1 read decode addr:{:04X}", addr),
        }
    }

    pub fn mbc3_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                let cart_addr = self.mbc3_rom_addr(addr);
                self.cartridge[cart_addr]
            }
            0xA000..=0xBFFF => self.mbc3_read_ram_or_rtc(addr),
            _ => unreachable!("Invalid mbc3 read decode addr:{:04X}", addr),
        }
    }

    fn mbc1_rom_addr(&self, addr: u16) -> usize {
        // TODO: Block accessing banks 0x20, 0x40, and 0x60
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

    fn mbc3_rom_addr(&self, addr: u16) -> usize {
        let offset = (addr as usize) & 0x3FFF;
        let bank = match addr {
            0x0000..=0x3FFF => 0,
            0x4000..=0x7FFF => self.mbc3_current_rom_bank(),
            _ => unreachable!("Invalid ROM decode addr:{:04X}", addr),
        };
        let rom_len = self.cartridge.len();
        debug_assert!(rom_len > 0, "cartridge must contain data");
        ((bank << 14) | offset) % rom_len
    }

    fn mbc3_current_rom_bank(&self) -> usize {
        let mut bank = (self.mbc3_rom_bank as usize) & 0x7F;
        if bank == 0 {
            bank = 1;
        }
        let total = self.total_rom_banks();
        if total == 0 {
            return 0;
        }
        let mut normalized = bank % total;
        if normalized == 0 && total > 1 {
            normalized = 1;
        }
        normalized
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
        self.total_rom_banks()
    }

    fn mbc1_active_ram_bank(&self) -> usize {
        if self.mbc1bankmode & 0x1 == 0x1 {
            (self.mbc1rambank & 0x3) as usize
        } else {
            0
        }
    }

    fn mbc3_active_ram_bank(&self) -> usize {
        (self.mbc3_ram_bank & 0x3) as usize
    }

    fn read_cart_ram_bank(&mut self, bank: usize, addr: u16) -> u8 {
        if self.cart_ram.is_empty() {
            return 0xFF;
        }

        let total = self.total_ram_banks();
        if total == 0 {
            return 0xFF;
        }

        let offset = (addr - 0xA000) as usize;
        let bank = bank % total;
        let index = bank * 0x2000 + (offset % 0x2000);
        let value = self.cart_ram[index];
        self.mem[addr as usize] = value;
        value
    }

    fn write_cart_ram_bank(&mut self, bank: usize, addr: u16, data: u8) {
        if self.cart_ram.is_empty() {
            return;
        }
        let total = self.total_ram_banks();
        if total == 0 {
            return;
        }
        let offset = (addr - 0xA000) as usize;
        let bank = bank % total;
        let index = bank * 0x2000 + (offset % 0x2000);
        if index < self.cart_ram.len() {
            self.cart_ram[index] = data;
            self.mem[addr as usize] = data;
        }
    }

    fn total_rom_banks(&self) -> usize {
        let banks = self.cartridge.len() / 0x4000;
        if banks == 0 { 1 } else { banks }
    }

    fn total_ram_banks(&self) -> usize {
        if self.cart_ram.is_empty() {
            0
        } else {
            self.cart_ram.len() / 0x2000
        }
    }

    fn mbc1_ram_write(&mut self, addr: u16, data: u8) {
        if !self.ram_enable {
            return;
        }
        self.write_cart_ram_bank(self.mbc1_active_ram_bank(), addr, data);
    }

    fn mbc3_ram_or_rtc_write(&mut self, addr: u16, data: u8) {
        if !self.ram_enable {
            return;
        }
        if let Some(reg) = self.mbc3_rtc_select {
            self.write_rtc_register(reg, data);
            return;
        }
        self.write_cart_ram_bank(self.mbc3_active_ram_bank(), addr, data);
    }

    fn mbc3_read_ram_or_rtc(&mut self, addr: u16) -> u8 {
        if !self.ram_enable {
            return 0xFF;
        }
        if let Some(reg) = self.mbc3_rtc_select {
            return self.read_rtc_register(reg);
        }
        self.read_cart_ram_bank(self.mbc3_active_ram_bank(), addr)
    }

    fn latch_rtc_data(&mut self, value: u8) {
        let value = value & 0x01;
        if self.rtc_latch_prev == 0 && value == 1 {
            self.update_rtc();
            self.rtc_latched = self.current_rtc_values();
            self.rtc_latch_active = true;
        } else if value == 0 {
            self.rtc_latch_active = false;
        }
        self.rtc_latch_prev = value;
    }

    fn read_rtc_register(&mut self, reg: u8) -> u8 {
        self.update_rtc();
        let snapshot = if self.rtc_latch_active {
            self.rtc_latched
        } else {
            self.current_rtc_values()
        };
        match reg {
            0x08..=0x0C => snapshot[(reg - 0x08) as usize],
            _ => 0xFF,
        }
    }

    fn write_rtc_register(&mut self, reg: u8, value: u8) {
        self.update_rtc();
        match reg {
            0x08 => self.rtc_seconds = value % 60,
            0x09 => self.rtc_minutes = value % 60,
            0x0A => self.rtc_hours = value % 24,
            0x0B => {
                let upper = self.rtc_day_counter & 0x100;
                self.rtc_day_counter = upper | (value as u16);
            }
            0x0C => {
                let lower = self.rtc_day_counter & 0xFF;
                let bit8 = (value as u16 & 0x01) << 8;
                self.rtc_day_counter = lower | bit8;
                self.rtc_halt = value & 0x40 != 0;
                self.rtc_day_carry = value & 0x80 != 0;
                if self.rtc_halt {
                    self.rtc_last_update = Instant::now();
                }
            }
            _ => (),
        }
    }

    fn current_rtc_values(&self) -> [u8; 5] {
        let day_low = (self.rtc_day_counter & 0xFF) as u8;
        let mut day_high = ((self.rtc_day_counter >> 8) & 0x1) as u8;
        if self.rtc_halt {
            day_high |= 0x40;
        }
        if self.rtc_day_carry {
            day_high |= 0x80;
        }
        [
            self.rtc_seconds.min(59),
            self.rtc_minutes.min(59),
            self.rtc_hours.min(23),
            day_low,
            day_high,
        ]
    }

    fn update_rtc(&mut self) {
        if self.rtc_halt {
            self.rtc_last_update = Instant::now();
            return;
        }

        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.rtc_last_update);
        let secs = elapsed.as_secs();
        if secs == 0 {
            return;
        }

        let total_seconds = self.rtc_seconds as u64 + secs;
        self.rtc_seconds = (total_seconds % 60) as u8;
        let mut carry = total_seconds / 60;

        if carry > 0 {
            let total_minutes = self.rtc_minutes as u64 + carry;
            self.rtc_minutes = (total_minutes % 60) as u8;
            carry = total_minutes / 60;
        }

        if carry > 0 {
            let total_hours = self.rtc_hours as u64 + carry;
            self.rtc_hours = (total_hours % 24) as u8;
            carry = total_hours / 24;
        }

        if carry > 0 {
            let mut days = self.rtc_day_counter as u64 + carry;
            if days >= 512 {
                self.rtc_day_carry = true;
                days %= 512;
            }
            self.rtc_day_counter = days as u16;
        }

        self.rtc_last_update = self
            .rtc_last_update
            .checked_add(Duration::from_secs(secs))
            .unwrap_or(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_mbc3_test_rom(bank_count: usize) -> Vec<u8> {
        assert!(bank_count >= 2, "need at least two banks for testing");
        let mut rom = vec![0u8; bank_count * 0x4000];
        for bank in 0..bank_count {
            let fill = bank as u8;
            let start = bank * 0x4000;
            rom[start..start + 0x4000].fill(fill);
        }
        rom[CART_TYPE] = 0x11; // MBC3
        rom[CART_SIZE] = 0x01; // 4 banks (64 KiB)
        rom[CART_RAM] = 0x03; // 32 KiB RAM (unused but realistic)
        rom
    }

    #[test]
    fn mbc3_switches_rom_banks() {
        let rom = build_mbc3_test_rom(4);
        let mut mem = Memory::new(&rom);

        // Initial bank at 0x4000 should be bank 1 (value 0x01)
        mem.set_addr(0x4000);
        mem.read();
        assert_eq!(mem.data(), 0x01);

        // Switch to bank 2 and verify reads reflect the change
        mem.set_addr(0x2000);
        mem.set_data(0x02);
        mem.write();

        mem.set_addr(0x4000);
        mem.read();
        assert_eq!(mem.data(), 0x02);
    }

    #[test]
    fn mbc3_ram_banking() {
        let rom = build_mbc3_test_rom(4);
        let mut mem = Memory::new(&rom);

        // Enable RAM
        mem.set_addr(0x0000);
        mem.set_data(0x0A);
        mem.write();

        // Select RAM bank 1
        mem.set_addr(0x4000);
        mem.set_data(0x01);
        mem.write();

        // Write a value into bank 1
        mem.set_addr(0xA000);
        mem.set_data(0x77);
        mem.write();

        // Switch to RAM bank 2 and write a different value
        mem.set_addr(0x4000);
        mem.set_data(0x02);
        mem.write();

        mem.set_addr(0xA000);
        mem.set_data(0x99);
        mem.write();

        // Read back from bank 2 (should be 0x99)
        mem.set_addr(0xA000);
        mem.read();
        assert_eq!(mem.data(), 0x99);

        // Switch back to bank 1 and ensure original value is intact
        mem.set_addr(0x4000);
        mem.set_data(0x01);
        mem.write();

        mem.set_addr(0xA000);
        mem.read();
        assert_eq!(mem.data(), 0x77);
    }
}
