SECTION "Header", ROM0[$100]
  jp EntryPoint
  ds $150 - @, 0

EntryPoint:
  di
  ld de, 0x0000
  xor a
  ld [$FF0F], a      ; clear IF
  ld [$FF06], a      ; reset TMA
  ld [$FF04], a      ; reset DIV
  ld a, 0x5          ; enable timer, input clock 16384 Hz
  ld [$FF07], a
  ld a, $04          ; enable timer interrupt
  ld [$FFFF], a
  ei

Loop:
  inc de
  jr Loop

SECTION "TimerHandler", ROM0[$50]
TimerHandler:
  halt

