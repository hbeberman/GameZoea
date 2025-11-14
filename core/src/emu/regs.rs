// Joypad
pub const P1: u16 = 0xFF00;

// Timer
pub const DIV: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA: u16 = 0xFF06;
pub const TAC: u16 = 0xFF07;

// PPU
pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY: u16 = 0xFF42;
pub const SCX: u16 = 0xFF43;
pub const LY: u16 = 0xFF44;
pub const LYC: u16 = 0xFF45;
pub const DMA: u16 = 0xFF46;
pub const BGP: u16 = 0xFF47;
pub const WY: u16 = 0xFF4A;
pub const WX: u16 = 0xFF4B; // TODO: pandocs say WX0 and WX116 are weird

//Interrupts
pub const IF: u16 = 0xFF0F;
pub const IE: u16 = 0xFFFF;

// Cartridge
pub const CART_TYPE: usize = 0x147;
pub const CART_SIZE: usize = 0x148;
pub const CART_RAM: usize = 0x149;
