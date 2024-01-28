use super::cartridge::Rom;
use super::ppu::NesPPU;

// #[derive(Clone)]
pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    rom: Rom,
    pub ppu: NesPPU,

    pub cycles: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU) + 'call>,
}

impl<'call> Bus<'call> {
    pub fn new<F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&NesPPU) + 'call,
        {


        let mut cpu_vram: [u8; 2048] = [0; 2048];
        let cpu_vram_len = cpu_vram.len();
        let rom_prg_len = rom.prg_rom.len();
        /*if rom_prg_len > cpu_vram_len {
            cpu_vram.copy_from_slice(&rom.prg_rom[..cpu_vram_len]);
        } else {
            cpu_vram.copy_from_slice(&rom.prg_rom[..rom_prg_len]);
        }*/

        let ppu = NesPPU::new(rom.chr_rom.clone(), rom.screen_mirroring);
        Bus {
            cpu_vram,
            rom,
            ppu,
            // cycles is 7 is taken from nestest.log
            cycles:7,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror if needed
            addr = addr % 0x4000;
        }

        self.rom.prg_rom[addr as usize]
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        let new_frame = self.ppu.tick(cycles * 3);
        if new_frame {
            (self.gameloop_callback)(&self.ppu);
        }

        /*
        self.cycles += cycles as usize;
        let mut nmi_before:bool = false;
        if Some(true) == self.ppu.nmi_interrupt {
            nmi_before = true;
        }
        self.ppu.tick(cycles * 3);

        let mut nmi_after:bool = false;
        if Some(true) == self.ppu.nmi_interrupt {
            nmi_after = true;
        }

        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&self.ppu);
        }*/
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        return self.ppu.pull_nmi_interrupt();
    }
}

pub trait Mem {
    fn mem_read(&mut self, addr: u16) -> u8;
    fn mem_write(&mut self, addr: u16, data: u8);
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;

const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl<'call> Mem for Bus<'call> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                0x0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_read(mirror_down_addr)
            }
            0x4000..=0x4015 => {
                //ignore APU 
                0
            }
            0x8000..=0xFFFF => self.read_prg_rom(addr),
            _ => {
                println!("Ignoring mem accesss at 0x{:02X}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => {
                panic!("status is read only");
            }
            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }
            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }

            0x2006 => {
                self.ppu.write_to_ppu_addr(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            0x8000..=0xFFFF => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);

                // todo: handle this eventually
                // let add_cycles: u16 = if self.cycles % 2 == 1 { 514 } else { 513 };
                // self.tick(add_cycles); //todo this will cause weird effects as PPU will have 513/514 * 3 ticks
            }
            _ => {
                panic!("Ignoring mem accesss at 0x{:02X}", addr);
            }
        }
    }
}
