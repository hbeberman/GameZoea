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
    // }}}
