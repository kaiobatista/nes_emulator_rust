use crate::ppu::PPU;
use crate::ines_file::Rom;

pub struct Bus {
    pub ram: [u8; 2 * 1024],
    pub rom: Rom,
    pub ppu: PPU,
}

impl Bus {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => {
                self.ram[(addr as usize) & 0x07FF as usize]
            }
            
            0x2000..=0x3FFF => {
                self.ppu.cpu_read(addr & 0x0007, false, &mut self.rom)
            }

            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let rom_size = self.rom.prg_rom.len() as u16;
                let index = (offset % rom_size) as usize;
                self.rom.prg_rom[index]
            }
            _ => {0}
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr as usize {
            0x0000..=0x1FFF => {
                self.ram[(addr as usize) & 0x07FF as usize] = data;
            }

            0x2000..=0x3FFF => {
                self.ppu.cpu_write(addr & 0x0007, data, &mut self.rom);
            }

            0x8000..=0xFFFF => {
                let offset = addr - 0x8000;
                let rom_size = self.rom.prg_rom.len() as u16;
                let index = (offset % rom_size) as usize;
                self.rom.prg_rom[index] = data;
            }
            _ => {}
        }
    }
}
