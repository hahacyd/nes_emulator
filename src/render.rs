pub mod frame;
pub mod palette;

use crate::ppu::NesPPU;
use frame::Frame;
use frame::show_tile;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.bknd_pattern_addr();
    for i in 0..0x03c0 {
        let tile_idx = ppu.vram[i] as usize;
        let tile_x = i % 32;
        let tile_y = i / 32;
        show_tile(&ppu.chr_rom, frame, bank, tile_x, tile_y, tile_idx);
    }
}
