use super::palette;

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

pub fn show_frame(chr_rom: &Vec<u8>) -> Frame {
    let mut frame = Frame::new();

    for i in 0..(Frame::WIDTH * Frame::HEIGHT / 64) {
        let tile_x = i % 32;
        let tile_y = i / 32;
        show_tile(chr_rom, &mut frame, 0, tile_x, tile_y, i);
    }
    frame
}

pub fn show_tile(chr_rom: &Vec<u8>, frame: &mut Frame, bank: u16, tile_x:usize, tile_y:usize, tile_idx: usize) {
    assert!(bank <= 1);
    if tile_idx < chr_rom.len() {
        return;
    }

    let bank = (bank * 0x1000) as usize;

    let tile = &chr_rom[(bank + tile_idx * 16)..=(bank + tile_idx * 16 + 15)];
    
    for y in 0..=7 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..=7).rev() {
            let value = (1 & upper) << 1 | (1 & lower);
            upper = upper >> 1;
            lower = lower >> 1;

            let rgb = match value {
                0 => palette::SYSTEM_PALLETE[0x01],
                1 => palette::SYSTEM_PALLETE[0x23],
                2 => palette::SYSTEM_PALLETE[0x27],
                3 => palette::SYSTEM_PALLETE[0x30],
                _ => panic!("can't be"),
            };
            frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
        }
    }
}

