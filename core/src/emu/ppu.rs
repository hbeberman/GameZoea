use crate::app::window::{FrameSender, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::emu::gb::Comp;
use crate::emu::mem::Memory;
use crate::{bit, clearbit, isbitset, setbit};
use std::cell::RefCell;
use std::rc::Rc;

pub const BLACK: [u8; 4] = [0x29, 0x41, 0x39, 0xFF];
pub const DARK_GREY: [u8; 4] = [0x39, 0x59, 0x4A, 0xFF];
pub const LIGHT_GREY: [u8; 4] = [0x5A, 0x79, 0x42, 0xFF];
pub const WHITE: [u8; 4] = [0x7B, 0x82, 0x10, 0xFF];

const FRAME_BYTES: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize) * 4;

const IFLAG: u16 = 0xFF0F;
const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const BGP: u16 = 0xFF47;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B; // TODO: pandocs say WX0 and WX116 are weird

#[derive(Debug, PartialEq, Clone)]
enum Mode {
    M0,
    M1,
    M2,
    M3,
}

impl Mode {
    fn bits(&self) -> u8 {
        match self {
            Mode::M0 => 0,
            Mode::M1 => 1,
            Mode::M2 => 2,
            Mode::M3 => 3,
        }
    }

    fn next(&self) -> Self {
        match self {
            Mode::M0 => Mode::M1,
            Mode::M1 => Mode::M2,
            Mode::M2 => Mode::M3,
            Mode::M3 => Mode::M0,
        }
    }
}

#[derive(Debug)]
enum Fetch {
    Tile_,
    Tile,
    DataLo_,
    DataLo,
    DataHi_,
    DataHi,
    Sleep0_,
    Sleep1_,
    Push,
}

impl Fetch {
    fn next(&self) -> Self {
        match self {
            Fetch::Tile_ => Fetch::Tile,
            Fetch::Tile => Fetch::DataLo_,
            Fetch::DataLo_ => Fetch::DataLo,
            Fetch::DataLo => Fetch::DataHi_,
            Fetch::DataHi_ => Fetch::DataHi,
            Fetch::DataHi => Fetch::Sleep0_,
            Fetch::Sleep0_ => Fetch::Sleep1_,
            Fetch::Sleep1_ => Fetch::Push,
            Fetch::Push => Fetch::Tile,
        }
    }
}

type TileData = [u8; 16];

#[allow(dead_code)]
pub struct Ppu {
    frame_tx: Option<FrameSender>,
    mem: Rc<RefCell<Memory>>,
    bg_fifo: Vec<Pixel>,
    obj_fifo: Vec<Pixel>,
    x: u8,
    pub testing: usize,
    back_buffer: Vec<u8>,
    mode: Mode,
    dot: u16,
    dotlimit: u16,
    fetch_state: Fetch,
    fetch_tile: u8,
    fetch_tile_datalo: u8,
    fetch_tile_datahi: u8,
    lcd_was_enabled: bool, // Track LCD emut nable state
    already_interrupted: bool,
}

#[allow(dead_code)]
pub struct Pixel {
    color: u8, // 0..=3
    palette: u8,
    // sprite_priority: u8, CGB only
    bg_priority: u8,
}

impl Ppu {
    pub fn headless_dmg(mem: Rc<RefCell<Memory>>) -> Self {
        let mut ppu = Ppu {
            frame_tx: None,
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            testing: 0,
            back_buffer: vec![0; FRAME_BYTES],
            mode: Mode::M0,
            dot: 0x0000,
            dotlimit: 0x0000,
            fetch_state: Fetch::Tile_,
            fetch_tile: 0x00,
            fetch_tile_datalo: 0x00,
            fetch_tile_datahi: 0x00,
            lcd_was_enabled: false,
            already_interrupted: false,
        };

        ppu.mem_write(LCDC, 0x91);
        ppu.mem_write(STAT, 0x80);
        ppu.mem_write(SCY, 0x00);
        ppu.mem_write(SCX, 0x00);
        ppu.mem_write(LY, 0x00);
        ppu.mem_write(LYC, 0x00);
        ppu.mem_write(BGP, 0xFC);
        ppu.mem_write(WY, 0x00);
        ppu.mem_write(WX, 0x00);

        ppu
    }

    pub fn init_dmg(frame_tx: FrameSender, mem: Rc<RefCell<Memory>>) -> Self {
        let mut ppu = Ppu {
            frame_tx: Some(frame_tx),
            mem,
            bg_fifo: Vec::<Pixel>::new(),
            obj_fifo: Vec::<Pixel>::new(),
            x: 0,
            testing: 0,
            back_buffer: vec![0; FRAME_BYTES],
            mode: Mode::M0,
            dot: 0x0000,
            dotlimit: 0x0000,
            fetch_state: Fetch::Tile_,
            fetch_tile: 0x00,
            fetch_tile_datalo: 0x00,
            fetch_tile_datahi: 0x00,
            lcd_was_enabled: false,
            already_interrupted: false,
        };

        ppu.mem_write(LCDC, 0x91);
        ppu.mem_write(STAT, 0x80);
        ppu.mem_write(SCY, 0x00);
        ppu.mem_write(SCX, 0x00);
        ppu.mem_write(LY, 0x00);
        ppu.mem_write(LYC, 0x00);
        ppu.mem_write(BGP, 0xFC);
        ppu.mem_write(WY, 0x00);
        ppu.mem_write(WX, 0x00);

        ppu
    }

    pub fn tick(&mut self, t: u128) {
        let _ = t;
        let lcdc = self.mem_read(LCDC);
        let lcd_enabled = (lcdc & 0x80) != 0;

        if !lcd_enabled {
            self.lcd_was_enabled = false;
            return;
        }

        // Detect LCD being turned on - reset PPU state
        if !self.lcd_was_enabled {
            self.lcd_was_enabled = true;
            self.x = 0;
            self.set_ly(0);
            self.reset_fetch_pipeline();
            self.mode = Mode::M2;
            self.dot = 80;
            // TODO: Add a 5th color for LCD off
            for chunk in self.back_buffer.chunks_exact_mut(4) {
                chunk.copy_from_slice(&WHITE);
            }
        }

        let mode = self.mode.clone();
        match mode {
            Mode::M0 => self.hblank(),
            Mode::M1 => self.vblank(),
            Mode::M2 => self.oamscan(),
            Mode::M3 => self.draw(),
        }
        if mode != self.mode {
            self.set_mode(self.mode.bits());
            self.check_interrupt();
        }
        self.update_stat();
    }

    pub fn oamscan(&mut self) {
        self.dot -= 1;
        if self.dot == 0 {
            // Next mode is Drawing
            self.mode = Mode::M3;
            self.dot = 289;
            //           eprintln!("Entering Drawing mode:{:?} dot:#{}", self.mode, self.dot);
        }
    }

    pub fn hblank(&mut self) {
        self.dot -= 1;
        if self.dot == 0 {
            self.x = 0;
            let ly = self.ly() + 1;
            self.set_ly(ly);
            self.dot = if ly == 144 {
                // Next mode is VBLANK
                self.mode = Mode::M1;
                /*
                                eprintln!(
                                    "Entering VBLANK mode:{:?} dot:#{} ly:#{}",
                                    self.mode, self.dot, ly
                                );
                */
                let intflags = self.mem_read(IFLAG) | 0x1;
                self.mem_write(0xFF0F, intflags);
                456
            } else {
                // Next mode is OAM scan
                self.mode = Mode::M2;
                /*
                                eprintln!(
                                    "Entering OAM mode:{:?} dot:#{} ly:#{}",
                                    self.mode, self.dot, ly
                                );
                */
                self.reset_fetch_pipeline();
                80
            };
        }
    }

    pub fn draw(&mut self) {
        self.fifo_pixel_fetcher();
        self.render();
        self.dot -= 1;
        if u32::from(self.x) >= SCREEN_WIDTH {
            self.mode = Mode::M0;
            //eprintln!("Entering HBLANK mode:{:?} dot:#{}", self.mode, self.dot);
            self.dot += 87;
        }
    }

    pub fn vblank(&mut self) {
        self.dot -= 1;
        if self.dot == 0 {
            let ly = self.ly();
            if ly >= 153 {
                // Next mode is OAM
                self.mode = Mode::M2;
                //eprintln!("Entering OAM mode:{:?} dot:#{}", self.mode, self.dot);
                self.dot = 80;
                self.set_ly(0);
                self.x = 0;
                self.reset_fetch_pipeline();
                let Some(frame_tx) = &self.frame_tx else {
                    return;
                };

                if let Err(err) = frame_tx.send(self.back_buffer.clone()) {
                    eprintln!("failed to deliver frame: {err}");
                }
            } else {
                self.set_ly(ly.wrapping_add(1));
                self.dot = 456;
            }
        }
    }

    pub fn render(&mut self) {
        let pixel = if self.bg_fifo.is_empty() {
            return;
        } else {
            self.bg_fifo.remove(0)
        };

        let index = self.x as usize + self.ly() as usize * SCREEN_WIDTH as usize;
        if let Some(target) = self.back_buffer.get_mut((index * 4)..((index + 1) * 4)) {
            target.copy_from_slice(&Ppu::get_color(pixel.color));
        }
        self.x += 1;
    }

    fn reset_fetch_pipeline(&mut self) {
        self.bg_fifo.clear();
        self.obj_fifo.clear();
        self.fetch_state = Fetch::Tile_;
        self.fetch_tile = 0x00;
        self.fetch_tile_datalo = 0x00;
        self.fetch_tile_datahi = 0x00;
    }

    //
    pub fn tile_address_lo(&self, obj: bool, id: u8, y: u8) -> u16 {
        let lcdc = self.mem_read(LCDC);
        if obj || isbitset!(lcdc, 4) {
            0x8000 + ((id as u16) * 16) + ((y as u16) % 0x8) * 2
        } else {
            match id {
                0..128 => 0x9000 + ((id as u16) * 16) + ((y as u16) % 0x8) * 2,
                128..=255 => 0x8800 + (((id - 128) as u16) * 16) + ((y as u16) % 0x8) * 2,
            }
        }
    }

    pub fn read_whole_tile_data(&mut self, obj: bool, id: u8, _bank: u8) -> TileData {
        // TODO: Check if PPU access to VRAM is blocked, if so return 0xFF

        let lcdc = self.mem_read(LCDC);
        let addr = if obj || isbitset!(lcdc, 4) {
            0x8000 + ((id as u16) * 16)
        } else {
            match id {
                0..128 => 0x9000 + ((id as u16) * 16),
                128..=255 => 0x8800 + (((id - 128) as u16) * 16),
            }
        };
        let data = self.mem_read_16(addr);
        println!(
            "id:{:02X} tiledata: {}",
            id,
            data.iter()
                .map(|b| format!(" {:02x}", b))
                .collect::<String>()
        );
        data
    }

    pub fn read_tile(&mut self, x: u8, y: u8) -> u8 {
        // TODO: Need to handle windowing and tilemapping

        let map = 0x00;
        match map {
            0x00 => self.mem_read(0x9800 + x as u16 + (y as u16) * 32),
            0x01 => self.mem_read(0x9C00 + x as u16 + (y as u16) * 32),
            _ => panic!("Invalid tile map value used!"),
        }
    }

    fn with_mem_mut<R>(&self, f: impl FnOnce(&mut Memory) -> R) -> R {
        let mut mem = self.mem.borrow_mut();
        f(&mut mem)
    }

    fn with_mem<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        let mem = self.mem.borrow();
        f(&mem)
    }

    pub fn mem_read(&self, addr: u16) -> u8 {
        self.with_mem(|mem| mem.dbg_read(addr))
    }

    pub fn mem_read_16(&self, addr: u16) -> [u8; 16] {
        self.with_mem(|mem| mem.dbg_read_16(addr))
    }

    pub fn mem_write(&mut self, addr: u16, data: u8) {
        self.with_mem_mut(|mem| mem.dbg_write(addr, data));
    }

    pub fn own(&mut self, own: bool) {
        let owner = if own { Comp::Ppu } else { Comp::None };
        self.with_mem_mut(|mem| mem.set_owner(owner))
    }

    pub fn set_mode(&mut self, mode: u8) {
        let mut stat = self.mem_read(STAT);
        stat = (stat & 0xFC) | (mode & 0x3);
        self.mem_write(STAT, stat);
    }

    pub fn mode(&self) -> u8 {
        self.mem_read(STAT) & 0x3
    }

    pub fn ly(&self) -> u8 {
        self.mem_read(0xFF44)
    }

    pub fn set_ly(&mut self, ly: u8) {
        self.mem_write(0xFF44, ly)
    }

    pub fn lyc(&self) -> u8 {
        self.mem_read(0xFF45)
    }

    pub fn set_lyc(&mut self, lyc: u8) {
        self.mem_write(0xFF45, lyc)
    }

    pub fn stat(&self) -> u8 {
        self.mem_read(0xFF41)
    }

    pub fn update_stat(&mut self) {
        let mut stat = self.mem_read(STAT);
        if self.ly() == self.lyc() {
            stat &= bit!(2);
        } else {
            stat &= !(bit!(2));
        }
        self.mem_write(STAT, stat);
    }

    pub fn check_interrupt(&mut self) {
        let stat = self.mem_read(STAT);
        let hit = match self.mode {
            Mode::M0 => isbitset!(stat, 2),
            Mode::M1 => isbitset!(stat, 3),
            Mode::M2 => isbitset!(stat, 4),
            Mode::M3 => isbitset!(stat, 5),
        };
        if hit {
            if !self.already_interrupted {
                self.already_interrupted = true;
                let mut reg_if = self.mem_read(IFLAG);
                setbit!(reg_if, 2);
                self.mem_write(IFLAG, reg_if);
            }
        } else {
            self.already_interrupted = false;
        }
    }

    pub fn fifo_pixel_fetcher(&mut self) {
        let x = self.x;
        let y = self.ly();

        match self.fetch_state {
            Fetch::Tile => {
                let scx = self.mem_read(SCX);
                let scy = self.mem_read(SCY);

                let screen_x = x.wrapping_add(self.bg_fifo.len() as u8);
                let bg_x = screen_x.wrapping_add(scx);
                let bg_y = y.wrapping_add(scy);

                let tile_x = (bg_x / 8) % 32;
                let tile_y = (bg_y / 8) % 32;

                self.fetch_tile = self.read_tile(tile_x, tile_y);
            }
            Fetch::DataLo => {
                let scy = self.mem_read(SCY);
                let tile_row = y.wrapping_add(scy) % 8;
                let addr = self.tile_address_lo(false, self.fetch_tile, tile_row);
                self.fetch_tile_datalo = self.mem_read(addr);
            }
            Fetch::DataHi => {
                let scy = self.mem_read(SCY);
                let tile_row = y.wrapping_add(scy) % 8;
                let addr = self.tile_address_lo(false, self.fetch_tile, tile_row) + 1;
                self.fetch_tile_datahi = self.mem_read(addr);
            }
            Fetch::Push => {
                if self.bg_fifo.len() > 8 {
                    return;
                }

                for i in (0..8).rev() {
                    let lo = (self.fetch_tile_datalo >> i) & 0x1;
                    let hi = (self.fetch_tile_datahi >> i) & 0x1;
                    let color = self.palette_decode(lo + (hi << 1));
                    let pixel = Pixel {
                        color,
                        palette: 0,
                        bg_priority: 0,
                    };
                    self.bg_fifo.push(pixel);
                }
            }
            _ => (),
        }
        self.fetch_state = self.fetch_state.next();
    }

    pub fn get_color(index: u8) -> [u8; 4] {
        match index {
            0x0 => WHITE,
            0x1 => LIGHT_GREY,
            0x2 => DARK_GREY,
            0x3 => BLACK,
            _ => unreachable!("invalid color value"),
        }
    }

    pub fn palette_decode(&mut self, id: u8) -> u8 {
        let bgp = self.mem_read(BGP);
        match id {
            0x0 => bgp & 0x3,
            0x1 => (bgp >> 2) & 0x3,
            0x2 => (bgp >> 4) & 0x3,
            0x3 => (bgp >> 6) & 0x3,
            _ => unreachable!("invalid palette index"),
        }
    }
}
