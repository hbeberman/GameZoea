use crate::app::window::{FrameSender, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::emu::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub const BLACK: [u8; 4] = [0x29, 0x41, 0x39, 0xFF];
pub const DARK_GREY: [u8; 4] = [0x39, 0x59, 0x4A, 0xFF];
pub const LIGHT_GREY: [u8; 4] = [0x5A, 0x79, 0x42, 0xFF];
pub const WHITE: [u8; 4] = [0x7B, 0x82, 0x10, 0xFF];

const FRAME_BYTES: usize = (SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize) * 4;

const LCDC: u16 = 0xFF40;
const STAT: u16 = 0xFF41;
const SCY: u16 = 0xFF42;
const SCX: u16 = 0xFF43;
const LY: u16 = 0xFF44;
const LYC: u16 = 0xFF45;
const BGP: u16 = 0xFF47;
const WY: u16 = 0xFF4A;
const WX: u16 = 0xFF4B; // TODO: pandocs say WX0 and WX116 are weird

enum Mode {
    M0,
    M1,
    M2,
    M3,
}

impl Mode {
    fn next(&self) -> Self {
        match self {
            Mode::M0 => Mode::M1,
            Mode::M1 => Mode::M2,
            Mode::M2 => Mode::M3,
            Mode::M3 => Mode::M0,
        }
    }
}

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
    fetch_state: Fetch,
    fetch_tile: u8,
    fetch_tile_datalo: u8,
    fetch_tile_datahi: u8,
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
            fetch_state: Fetch::Tile_,
            fetch_tile: 0x00,
            fetch_tile_datalo: 0x00,
            fetch_tile_datahi: 0x00,
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
            fetch_state: Fetch::Tile_,

            fetch_tile: 0x00,
            fetch_tile_datalo: 0x00,
            fetch_tile_datahi: 0x00,
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

        self.fifo_pixel_fetcher();
        self.render();
        self.update_stat();
    }

    //
    pub fn tile_address_lo(&self, obj: bool, id: u8, y: u8) -> u16 {
        let lcdc = self.mem_read(LCDC);
        if obj || lcdc & (1 << 4) != 0 {
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
        let addr = if obj || lcdc & (1 << 4) != 0 {
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

    pub fn set_mode(&mut self, mode: u8) {
        self.mem_write(0xFF41, (self.mode() & 0xFC) & (mode & 0x3))
    }

    pub fn mode(&self) -> u8 {
        self.mem_read(0xFF41 & 0x3)
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

    pub fn set_stat_bit(&mut self, lyc: u8) {
        let mut stat = self.mem_read(0xFF41);
        stat &= 0x1 << lyc;
        self.mem_write(STAT, stat)
    }

    pub fn clear_stat_bit(&mut self, lyc: u8) {
        let mut stat = self.mem_read(0xFF41);
        stat &= !(0x1 << lyc);
        self.mem_write(STAT, stat)
    }

    pub fn update_stat(&mut self) {
        if self.ly() == self.lyc() {
            self.set_stat_bit(2);
        } else {
            self.clear_stat_bit(2);
        }
    }

    pub fn fifo_pixel_fetcher(&mut self) {
        let x = self.x;
        let y = self.ly();
        match self.fetch_state {
            Fetch::Tile => {
                let scx = self.mem_read(SCX);
                let scy = self.mem_read(SCY);
                // Calculate background pixel position with scroll
                let bg_x = x.wrapping_add(scx);
                let bg_y = y.wrapping_add(scy);
                // Convert to tile coordinates (divide by 8)
                let tile_x = (bg_x / 8) % 32;
                let tile_y = (bg_y / 8) % 32;

                self.fetch_tile = self.read_tile(tile_x, tile_y);
                eprintln!("fetchtile:{:02x}", self.fetch_tile);
            }
            Fetch::DataLo => {
                let scy = self.mem_read(SCY);
                let tile_row = y.wrapping_add(scy) % 8;
                self.fetch_tile_datalo =
                    self.mem_read(self.tile_address_lo(false, self.fetch_tile, tile_row));
            }
            Fetch::DataHi => {
                let scy = self.mem_read(SCY);
                let tile_row = y.wrapping_add(scy) % 8;
                self.fetch_tile_datahi =
                    self.mem_read(self.tile_address_lo(false, self.fetch_tile, tile_row) + 1);
            }
            Fetch::Push => {
                if !self.bg_fifo.is_empty() {
                    return;
                }

                for i in 0..8 {
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

    pub fn render(&mut self) {
        let pixel = match self.bg_fifo.pop() {
            Some(pixel) => pixel,
            None => return,
        };

        let index = self.x as usize + self.ly() as usize * SCREEN_WIDTH as usize;
        if let Some(target) = self.back_buffer.get_mut((index * 4)..((index + 1) * 4)) {
            target.copy_from_slice(&Ppu::get_color(pixel.color));
        }

        let mut frame_complete = false;
        self.x += 1;
        if self.x == SCREEN_WIDTH as u8 {
            self.x = 0;
            self.set_ly(self.ly() + 1);
            if self.ly() == SCREEN_HEIGHT as u8 {
                self.set_ly(0);
                frame_complete = true;
            }
        }

        if !frame_complete {
            return;
        }

        let Some(frame_tx) = &self.frame_tx else {
            return;
        };

        if let Err(err) = frame_tx.send(self.back_buffer.clone()) {
            eprintln!("failed to deliver frame: {err}");
        }
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
