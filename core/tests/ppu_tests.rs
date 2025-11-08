use gamezoea::emu::gb::*;
use macros::*;
macro_rules! assert_hex_eq {
    ($a:expr, $b:expr) => {
        assert!($a == $b, "assertion failed: {:#06x} != {:#06x}", $a, $b);
    };
}

    // {{{ test vram_writes
    #[test]
    fn vram_writes() {
        const ROM: &[u8] = gbasm! {r#"
    ; Disable LCD so VRAM can be safely written
    ld a, [$FF40]
    res 7, a
    ld [$FF40], a

    ; Write tile data directly to VRAM ($8000)
    ld hl, $8000
    ld a, $3C
    ld [hli], a
    ld a, $7E
    ld [hli], a
    ld a, $42
    ld [hli], a
    ld a, $42
    ld [hli], a

    ; Put tile 0 into top-left of background map ($9800)
    ld hl, $9800
    ld a, $00
    ld [hl], a

    ; Background palette (for contrast)
    ld a, %11100100
    ld [$FF47], a

    ; Enable LCD:
    ; bit7 = LCD on
    ; bit4 = use $8000 tile data
    ; bit0 = background on
    ld a, %10010001
    ld [$FF40], a

Forever:
    ld a, [$8000]   ; read from tile 0 start
    jr Forever      ; repeat
    "#};
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(80000);
        assert_hex_eq!(gb.cpu.mem_dbg_read(0x8000), 0x3C);
        assert_hex_eq!(gb.cpu.a(), 0x3C);
        let td = gb.ppu.read_whole_tile_data(false, 0x00,0x00);
        assert_hex_eq!(td[0], 0x3C);
        assert_hex_eq!(td[1], 0x7E);
        assert_hex_eq!(td[2], 0x42);
        assert_hex_eq!(td[3], 0x42);
    }

    // {{{ test vram_writes
    #[test]
    fn ppu_tile_read_lcdc4() {
        const ROM: &[u8] = gbasm! {r#"
  def rLCDC equ $FF40
  def rBGP equ $FF47
  ; Disable LCD before writing VRAM
  ld a, [rLCDC]
  res 7, a
  ld [rLCDC], a

  ;-----------------------------------------
  ; Copy tile data (3 tiles total)
  ;-----------------------------------------

  ; Tile 0 → $9000 (solid square)
  ld hl, $9000
  ld de, TileSolid
  ld bc, 16
.copySolid:
  ld a, [de]
  ld [hli], a
  inc de
  dec bc
  ld a, b
  or c
  jr nz, .copySolid

  ; Tile 1 → $9010 (checker pattern)
  ld hl, $9010
  ld de, TileChecker
  ld bc, 16
.copyChecker:
  ld a, [de]
  ld [hli], a
  inc de
  dec bc
  ld a, b
  or c
  jr nz, .copyChecker

  ; Tile -1 → $8FF0 (hollow box)
  ld hl, $8FF0
  ld de, TileHollow
  ld bc, 16
.copyHollow:
  ld a, [de]
  ld [hli], a
  inc de
  dec bc
  ld a, b
  or c
  jr nz, .copyHollow

  ;-----------------------------------------
  ; Fill BG map ($9800)
  ;-----------------------------------------
  ld hl, $9800
  ld a, $00
  ld [hli], a    ; tile 0 → $9000
  ld a, $01
  ld [hli], a    ; tile 1 → $9010
  ld a, $FF
  ld [hli], a    ; tile -1 → $8FF0

  ;-----------------------------------------
  ; Set palette and enable LCDC.4=0 mode
  ;-----------------------------------------
  ld a, $E4
  ld [rBGP], a

  ; LCDC bits:
  ; bit7=LCD ON, bit4=0 → tile data at $8800 (signed mode)
  ; bit0=BG ON
  ld a, $81
  ld [rLCDC], a

.hang:
  jr .hang


;===================================================
; Tile Data Section (hex patterns)
;===================================================
SECTION "Tiles", ROM0

; Tile 0: Solid square
TileSolid:
  db $FF,$00,$FF,$00,$FF,$00,$FF,$00
  db $FF,$00,$FF,$00,$FF,$00,$FF,$00

; Tile 1: Checker pattern
TileChecker:
  db $AA,$55,$55,$AA,$AA,$55,$55,$AA
  db $AA,$55,$55,$AA,$AA,$55,$55,$AA

; Tile -1: Hollow box
TileHollow:
  db $FF,$22,$81,$FF,$81,$00,$81,$00
  db $81,$00,$81,$00,$81,$00,$FF,$00
    "#};
        let mut gb = Gameboy::headless_dmg(ROM);
        gb.step(80000);
        let td = gb.ppu.read_whole_tile_data(false, 0x00,0x00);
        assert_hex_eq!(td[0], 0xFF);
        assert_hex_eq!(td[1], 0x00);
        assert_hex_eq!(td[2], 0xFF);
        assert_hex_eq!(td[3], 0x00);
        let td = gb.ppu.read_whole_tile_data(false, 0x01,0x00);
        assert_hex_eq!(td[0], 0xAA);
        assert_hex_eq!(td[1], 0x55);
        assert_hex_eq!(td[2], 0x55);
        assert_hex_eq!(td[3], 0xAA);
        let td = gb.ppu.read_whole_tile_data(false, 0xFF,0x00);
        assert_hex_eq!(td[0], 0xFF);
        assert_hex_eq!(td[1], 0x22);
        assert_hex_eq!(td[2], 0x81);
        assert_hex_eq!(td[3], 0xFF);
    }

    // }}}
