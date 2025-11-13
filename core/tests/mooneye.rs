use gamezoea::emu::gb::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    const MOONEYE_STEPS: u128 = 80_000_000;

    fn load_mooneye_rom(path: &str) -> Vec<u8> {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let rom_path = PathBuf::from(manifest_dir).join(path);
        fs::read(&rom_path).unwrap_or_else(|e| {
            panic!(
                "Failed to read ROM '{}': {}",
                rom_path.display(),
                e
            )
        })
    }

    macro_rules! mooneye_test {
        ($(#[$meta:meta])* $name:ident, $path:literal) => {
            #[test]
            $(#[$meta])*
            fn $name() {
                let rom = load_mooneye_rom($path);
                let mut gb = Gameboy::headless_dmg(&rom);
                gb.step_mooneye(MOONEYE_STEPS);
            }
        };
    }

    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_add_sp_e_timing_gb,
        "tests/roms/mooneye/acceptance/add_sp_e_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_bits_mem_oam_gb,
        "tests/roms/mooneye/acceptance/bits/mem_oam.gb"
    );
    mooneye_test!(
        mooneye_acceptance_bits_reg_f_gb,
        "tests/roms/mooneye/acceptance/bits/reg_f.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_bits_unused_hwio_gs_gb,
        "tests/roms/mooneye/acceptance/bits/unused_hwio-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_div_s_gb,
        "tests/roms/mooneye/acceptance/boot_div-S.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_div_dmg0_gb,
        "tests/roms/mooneye/acceptance/boot_div-dmg0.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_div_dmgabcmgb_gb,
        "tests/roms/mooneye/acceptance/boot_div-dmgABCmgb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_div2_s_gb,
        "tests/roms/mooneye/acceptance/boot_div2-S.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_hwio_s_gb,
        "tests/roms/mooneye/acceptance/boot_hwio-S.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_hwio_dmg0_gb,
        "tests/roms/mooneye/acceptance/boot_hwio-dmg0.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_hwio_dmgabcmgb_gb,
        "tests/roms/mooneye/acceptance/boot_hwio-dmgABCmgb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_regs_dmg0_gb,
        "tests/roms/mooneye/acceptance/boot_regs-dmg0.gb"
    );
    mooneye_test!(
        mooneye_acceptance_boot_regs_dmgabc_gb,
        "tests/roms/mooneye/acceptance/boot_regs-dmgABC.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_regs_mgb_gb,
        "tests/roms/mooneye/acceptance/boot_regs-mgb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_regs_sgb_gb,
        "tests/roms/mooneye/acceptance/boot_regs-sgb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_boot_regs_sgb2_gb,
        "tests/roms/mooneye/acceptance/boot_regs-sgb2.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_call_cc_timing_gb,
        "tests/roms/mooneye/acceptance/call_cc_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_call_cc_timing2_gb,
        "tests/roms/mooneye/acceptance/call_cc_timing2.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_call_timing_gb,
        "tests/roms/mooneye/acceptance/call_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_call_timing2_gb,
        "tests/roms/mooneye/acceptance/call_timing2.gb"
    );
    mooneye_test!(
        mooneye_acceptance_di_timing_gs_gb,
        "tests/roms/mooneye/acceptance/di_timing-GS.gb"
    );
    mooneye_test!(
        mooneye_acceptance_div_timing_gb,
        "tests/roms/mooneye/acceptance/div_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ei_sequence_gb,
        "tests/roms/mooneye/acceptance/ei_sequence.gb"
    );
    mooneye_test!(
        mooneye_acceptance_ei_timing_gb,
        "tests/roms/mooneye/acceptance/ei_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_halt_ime0_ei_gb,
        "tests/roms/mooneye/acceptance/halt_ime0_ei.gb"
    );
    mooneye_test!(
        mooneye_acceptance_halt_ime0_nointr_timing_gb,
        "tests/roms/mooneye/acceptance/halt_ime0_nointr_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_halt_ime1_timing_gb,
        "tests/roms/mooneye/acceptance/halt_ime1_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_halt_ime1_timing2_gs_gb,
        "tests/roms/mooneye/acceptance/halt_ime1_timing2-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_if_ie_registers_gb,
        "tests/roms/mooneye/acceptance/if_ie_registers.gb"
    );
    mooneye_test!(
        mooneye_acceptance_instr_daa_gb,
        "tests/roms/mooneye/acceptance/instr/daa.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_interrupts_ie_push_gb,
        "tests/roms/mooneye/acceptance/interrupts/ie_push.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_intr_timing_gb,
        "tests/roms/mooneye/acceptance/intr_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_jp_cc_timing_gb,
        "tests/roms/mooneye/acceptance/jp_cc_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_jp_timing_gb,
        "tests/roms/mooneye/acceptance/jp_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ld_hl_sp_e_timing_gb,
        "tests/roms/mooneye/acceptance/ld_hl_sp_e_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_oam_dma_basic_gb,
        "tests/roms/mooneye/acceptance/oam_dma/basic.gb"
    );
    mooneye_test!(
        mooneye_acceptance_oam_dma_reg_read_gb,
        "tests/roms/mooneye/acceptance/oam_dma/reg_read.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_oam_dma_sources_gs_gb,
        "tests/roms/mooneye/acceptance/oam_dma/sources-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_oam_dma_restart_gb,
        "tests/roms/mooneye/acceptance/oam_dma_restart.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_oam_dma_start_gb,
        "tests/roms/mooneye/acceptance/oam_dma_start.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_oam_dma_timing_gb,
        "tests/roms/mooneye/acceptance/oam_dma_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_pop_timing_gb,
        "tests/roms/mooneye/acceptance/pop_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_hblank_ly_scx_timing_gs_gb,
        "tests/roms/mooneye/acceptance/ppu/hblank_ly_scx_timing-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_1_2_timing_gs_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_1_2_timing-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_2_0_timing_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_2_0_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_2_mode0_timing_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_2_mode0_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_2_mode0_timing_sprites_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_2_mode0_timing_sprites.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_2_mode3_timing_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_2_mode3_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_intr_2_oam_ok_timing_gb,
        "tests/roms/mooneye/acceptance/ppu/intr_2_oam_ok_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_lcdon_timing_gs_gb,
        "tests/roms/mooneye/acceptance/ppu/lcdon_timing-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_lcdon_write_timing_gs_gb,
        "tests/roms/mooneye/acceptance/ppu/lcdon_write_timing-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_stat_irq_blocking_gb,
        "tests/roms/mooneye/acceptance/ppu/stat_irq_blocking.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_stat_lyc_onoff_gb,
        "tests/roms/mooneye/acceptance/ppu/stat_lyc_onoff.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ppu_vblank_stat_intr_gs_gb,
        "tests/roms/mooneye/acceptance/ppu/vblank_stat_intr-GS.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_push_timing_gb,
        "tests/roms/mooneye/acceptance/push_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_rapid_di_ei_gb,
        "tests/roms/mooneye/acceptance/rapid_di_ei.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ret_cc_timing_gb,
        "tests/roms/mooneye/acceptance/ret_cc_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_ret_timing_gb,
        "tests/roms/mooneye/acceptance/ret_timing.gb"
    );
    mooneye_test!(
        mooneye_acceptance_reti_intr_timing_gb,
        "tests/roms/mooneye/acceptance/reti_intr_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_reti_timing_gb,
        "tests/roms/mooneye/acceptance/reti_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_rst_timing_gb,
        "tests/roms/mooneye/acceptance/rst_timing.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_serial_boot_sclk_align_dmgabcmgb_gb,
        "tests/roms/mooneye/acceptance/serial/boot_sclk_align-dmgABCmgb.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_div_write_gb,
        "tests/roms/mooneye/acceptance/timer/div_write.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_rapid_toggle_gb,
        "tests/roms/mooneye/acceptance/timer/rapid_toggle.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim00_gb,
        "tests/roms/mooneye/acceptance/timer/tim00.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim00_div_trigger_gb,
        "tests/roms/mooneye/acceptance/timer/tim00_div_trigger.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim01_gb,
        "tests/roms/mooneye/acceptance/timer/tim01.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim01_div_trigger_gb,
        "tests/roms/mooneye/acceptance/timer/tim01_div_trigger.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim10_gb,
        "tests/roms/mooneye/acceptance/timer/tim10.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim10_div_trigger_gb,
        "tests/roms/mooneye/acceptance/timer/tim10_div_trigger.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim11_gb,
        "tests/roms/mooneye/acceptance/timer/tim11.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tim11_div_trigger_gb,
        "tests/roms/mooneye/acceptance/timer/tim11_div_trigger.gb"
    );
    mooneye_test!(
        mooneye_acceptance_timer_tima_reload_gb,
        "tests/roms/mooneye/acceptance/timer/tima_reload.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_timer_tima_write_reloading_gb,
        "tests/roms/mooneye/acceptance/timer/tima_write_reloading.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_acceptance_timer_tma_write_reloading_gb,
        "tests/roms/mooneye/acceptance/timer/tma_write_reloading.gb"
    );
    mooneye_test!(
        mooneye_emulator_only_mbc1_bits_bank1_gb,
        "tests/roms/mooneye/emulator-only/mbc1/bits_bank1.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_bits_bank2_gb,
        "tests/roms/mooneye/emulator-only/mbc1/bits_bank2.gb"
    );
    mooneye_test!(
        mooneye_emulator_only_mbc1_bits_mode_gb,
        "tests/roms/mooneye/emulator-only/mbc1/bits_mode.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_bits_ramg_gb,
        "tests/roms/mooneye/emulator-only/mbc1/bits_ramg.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_multicart_rom_8mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/multicart_rom_8Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_ram_256kb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/ram_256kb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_ram_64kb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/ram_64kb.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_16mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_16Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_1mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_1Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_2mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_2Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_4mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_4Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_512kb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_512kb.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc1_rom_8mb_gb,
        "tests/roms/mooneye/emulator-only/mbc1/rom_8Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_bits_ramg_gb,
        "tests/roms/mooneye/emulator-only/mbc2/bits_ramg.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_bits_romb_gb,
        "tests/roms/mooneye/emulator-only/mbc2/bits_romb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_bits_unused_gb,
        "tests/roms/mooneye/emulator-only/mbc2/bits_unused.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_ram_gb,
        "tests/roms/mooneye/emulator-only/mbc2/ram.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_rom_1mb_gb,
        "tests/roms/mooneye/emulator-only/mbc2/rom_1Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_rom_2mb_gb,
        "tests/roms/mooneye/emulator-only/mbc2/rom_2Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc2_rom_512kb_gb,
        "tests/roms/mooneye/emulator-only/mbc2/rom_512kb.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_16mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_16Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_1mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_1Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_2mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_2Mb.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_32mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_32Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_4mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_4Mb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_512kb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_512kb.gb"
    );
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_64mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_64Mb.gb"
    );
    */
    /*
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_emulator_only_mbc5_rom_8mb_gb,
        "tests/roms/mooneye/emulator-only/mbc5/rom_8Mb.gb"
    );
    */
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_madness_mgb_oam_dma_halt_sprites_gb,
        "tests/roms/mooneye/madness/mgb_oam_dma_halt_sprites.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_bits_unused_hwio_c_gb,
        "tests/roms/mooneye/misc/bits/unused_hwio-C.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_div_a_gb,
        "tests/roms/mooneye/misc/boot_div-A.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_div_cgb0_gb,
        "tests/roms/mooneye/misc/boot_div-cgb0.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_div_cgbabcde_gb,
        "tests/roms/mooneye/misc/boot_div-cgbABCDE.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_hwio_c_gb,
        "tests/roms/mooneye/misc/boot_hwio-C.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_regs_a_gb,
        "tests/roms/mooneye/misc/boot_regs-A.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_boot_regs_cgb_gb,
        "tests/roms/mooneye/misc/boot_regs-cgb.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_misc_ppu_vblank_stat_intr_c_gb,
        "tests/roms/mooneye/misc/ppu/vblank_stat_intr-C.gb"
    );
    mooneye_test!(
        #[ignore = "TODO"]
        mooneye_utils_bootrom_dumper_gb,
        "tests/roms/mooneye/utils/bootrom_dumper.gb"
    );
    mooneye_test!(
        mooneye_utils_dump_boot_hwio_gb,
        "tests/roms/mooneye/utils/dump_boot_hwio.gb"
    );
}
