use super::palette;
use crate::ppu::NesPPU;

pub struct Frame {
    pub data:Vec<u8>,
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; Frame::WIDTH * Frame::HEIGHT * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}

pub fn show_frame(ppu:&NesPPU) -> Frame {
    let mut frame = Frame::new();

    for i in 0..(Frame::WIDTH * Frame::HEIGHT / 64) {
        let tile_x = i % 32;
        let tile_y = i / 32;
        show_tile(&ppu, &mut frame, 0, tile_x, tile_y, i % 512);
    }
    frame
}

fn bg_pallette(ppu: &NesPPU, tile_y:usize, tile_x:usize) -> [u8; 4]{
    let attr_table_idx = tile_x / 4 + tile_y / 4 * 8;
    let attr = ppu.vram[0x3c0 + attr_table_idx];
    let pallet_idx = match (tile_x % 4 / 2, tile_y % 4 / 2) {
        (0, 0) => attr & 0b11,
        (1, 0) => (attr >> 2) & 0b11,
        (0, 1) => (attr >> 4) & 0b11,
        (1, 1) => (attr >> 6) & 0b11,
        (_, _) => panic!("should not happen."),
    };
    let pallete_start: usize = 1 + (pallet_idx as usize) * 4;
    [ppu.palette_table[0], ppu.palette_table[pallete_start], ppu.palette_table[pallete_start + 1], ppu.palette_table[pallete_start + 2]]
}

pub fn show_tile(ppu:&NesPPU, frame: &mut Frame, bank: u16, tile_x:usize, tile_y:usize, tile_idx: usize) {
    assert!(bank <= 1);

    let bank = (bank * 0x1000) as usize;

    let tile = &ppu.chr_rom[(bank + tile_idx * 16)..=(bank + tile_idx * 16 + 15)];
    let pallette = bg_pallette(&ppu, tile_y, tile_x);

    for y in 0..=7 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..=7).rev() {
            let value = (1 & upper) << 1 | (1 & lower);
            upper = upper >> 1;
            lower = lower >> 1;

            let rgb = match value {
                0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                1 => palette::SYSTEM_PALLETE[pallette[1] as usize],
                2 => palette::SYSTEM_PALLETE[pallette[2] as usize],
                3 => palette::SYSTEM_PALLETE[pallette[3] as usize],
                _ => panic!("can't be"),
            };
            frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
        }
    }
}

