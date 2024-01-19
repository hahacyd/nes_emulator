use bitflags::bitflags;

use crate::{cartridge::Mirroring, pallete};

#[derive(Clone)]
pub struct NesPPU {
    pub mirroring: Mirroring,

    // [0x3F00, 0x4000]
    pub palette_table: [u8; 32],

    // register
    pub ctrl: ControlRegister,
    pub mask:u8,
    pub status: Status,
    pub oam_addr: u8,

    pub oam_data: [u8; 256],
    pub scroll: u8,
    addr: AddrRegister,
    pub oam_dma: u8,

    // [0x2000, 3F00)
    pub vram: [u8; 2048],

    // [0x0000, 0x2000)
    pub chr_rom: Vec<u8>,

    // internal member
    internal_data_buf: u8,


    cycles: usize,
    scanline: usize,

    pub nmi_interrupt:Option<bool>,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom,
            mirroring,
            vram: [0; 2048],
            oam_dma: 0,
            palette_table: [0; 32],
            addr: AddrRegister::new(),
            ctrl: ControlRegister::new(),
            internal_data_buf: 0,
            oam_addr: 0,
            status: Status::new(),
            scroll: 0,
            oam_data: [0; 256],
            mask: 0,
            cycles: 0,
            scanline: 0,
            nmi_interrupt: Some(false),
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                if self.ctrl.contains(ControlRegister::GENERATE_NMI) {
                    self.status.set(Status::V_BLANK, true);
                    self.nmi_interrupt = Some(true);
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status.set(Status::V_BLANK, false);
                return true;
            }
        }
        return false;
    }

    pub fn read_status(&self) -> u8 {
        return self.status.bits();
    }

    pub fn read_oam_data(&self) -> u8 {
        return self.oam_data[self.oam_addr as usize];
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    //...  
 
   // Horizontal:
   //   [ A ] [ a ]
   //   [ B ] [ b ]
 
   // Vertical:
   //   [ A ] [ B ]
   //   [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10_1111_1111_1111;
        let vram_index = mirrored_vram - 0x2000;
        let name_table = vram_index / 0x400;
        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask = value;
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll = value;
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
    }

    pub fn write_to_ctrl(&mut self, value: u8) {
        self.ctrl.update(value);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => panic!("unexpected write to chr_rom {}", addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3eff => panic!("addr space 0x3000..0x3eff is not expected to be used, requested = {} ", addr),
            0x3f00..=0x3fff => panic!("addr is not expected to be used, {}", addr),
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.ctrl.vram_addr_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                return result;
            }
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => todo!("addr space 0x3000..0x3eff is not expected to be used, requested = {} ", addr),
            0x3f00..=0x3fff => {
                self.palette_table[(addr - 0x3f00) as usize]
            }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }
}

#[derive(Clone)]
pub struct AddrRegister {
    // hi , lo
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            value: (0, 0),
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.1 = (data & 0xff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        if self.get() > 0x3fff { // mirror down addr above 0x3fff
            self.set(self.get() & 0x3fff);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let lo = self.value.1;
        self.value.1 = self.value.1.wrapping_add(inc);
        if lo > self.value.1 {
            self.value.0 = self.value.0.wrapping_add(1);
        }

        if self.get() > 0x3fff {
            self.set(self.get() & 0x3fff);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) | (self.value.1 as u16)
    }
}

bitflags! {
    pub struct Status: u8 {
        const V_BLANK = 0b1000_0000;
        const SPRITE_0_HIT = 0b0100_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}

bitflags! {
    // 7  bit  0
    // ---- ----
    // VPHB SINN
    // |||| ||||
    // |||| ||++- Base nametable address
    // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
    // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
    // |||| |     (0: add 1, going across; 1: add 32, going down)
    // |||| +---- Sprite pattern table address for 8x8 sprites
    // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
    // |||+------ Background pattern table address (0: $0000; 1: $1000)
    // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
    // |+-------- PPU master/slave select
    // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
    // +--------- Generate an NMI at the start of the
    //            vertical blanking interval (0: off; 1: on)
    pub struct ControlRegister: u8 {
        const NAMETABLE1              = 0b0000_0001;
        const NAMETABLE2              = 0b0000_0010;
        const VRAM_ADD_INCREMENT      = 0b0000_0100;
        const SPRITE_PATTERN_ADDR     = 0b0000_1000;
        const BACKROUND_PATTERN_ADDR  = 0b0001_0000;
        const SPRITE_SIZE             = 0b0010_0000;
        const MASTER_SLAVE_SELECT     = 0b0100_0000;
        const GENERATE_NMI            = 0b1000_0000;
    }
}

impl Status {
    pub fn new() -> Self {
        Status::from_bits_truncate(0b0000_0000)
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}

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
        show_tile(chr_rom, &mut frame, 0, i);
    }
    frame
}

pub fn show_tile(chr_rom: &Vec<u8>, frame: &mut Frame, bank: usize, tile_n:usize) {
    assert!(bank <= 1);
    assert!(tile_n < (Frame::WIDTH * Frame::HEIGHT / 64));
    if tile_n >= chr_rom.len() / 16 {
        return;
    }

    let tile_x = tile_n % 32;
    let tile_y = tile_n / 32;

    let bank = (bank * 0x1000) as usize;

    let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];
    
    for y in 0..=7 {
        let mut upper = tile[y];
        let mut lower = tile[y + 8];

        for x in (0..=7).rev() {
            let value = (1 & upper) << 1 | (1 & lower);
            upper = upper >> 1;
            lower = lower >> 1;

            let rgb = match value {
                0 => pallete::SYSTEM_PALLETE[0x01],
                1 => pallete::SYSTEM_PALLETE[0x23],
                2 => pallete::SYSTEM_PALLETE[0x27],
                3 => pallete::SYSTEM_PALLETE[0x30],
                _ => panic!("can't be"),
            };
            frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
        }
    }
}
