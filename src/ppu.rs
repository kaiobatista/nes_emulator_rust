use crate::ines_file::Rom;
use sdl2::pixels::Color;


const NES_PALETTE: [Color; 64] = [
    Color::RGB(84, 84, 84), Color::RGB(0, 30, 116), Color::RGB(8, 16, 144), Color::RGB(48, 0, 136),
    Color::RGB(68, 0, 100), Color::RGB(92, 0, 48), Color::RGB(84, 4, 0), Color::RGB(68, 24, 0),
    Color::RGB(32, 42, 0), Color::RGB(8, 58, 0), Color::RGB(0, 64, 0), Color::RGB(0, 60, 0),
    Color::RGB(0, 50, 60), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0),
    Color::RGB(152, 150, 152), Color::RGB(0, 90, 180), Color::RGB(50, 78, 180), Color::RGB(100, 60, 172),
    Color::RGB(138, 44, 140), Color::RGB(164, 30, 80), Color::RGB(160, 44, 4), Color::RGB(138, 70, 0),
    Color::RGB(88, 104, 0), Color::RGB(32, 120, 0), Color::RGB(0, 128, 0), Color::RGB(0, 124, 40),
    Color::RGB(0, 110, 120), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0),
    Color::RGB(236, 238, 236), Color::RGB(0, 156, 236), Color::RGB(98, 140, 236), Color::RGB(160, 112, 236),
    Color::RGB(212, 92, 204), Color::RGB(236, 82, 120), Color::RGB(236, 94, 0), Color::RGB(212, 134, 0),
    Color::RGB(152, 180, 0), Color::RGB(80, 200, 0), Color::RGB(48, 216, 0), Color::RGB(48, 208, 70),
    Color::RGB(48, 180, 200), Color::RGB(72, 72, 72), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0),
    Color::RGB(236, 238, 236), Color::RGB(168, 204, 236), Color::RGB(188, 188, 236), Color::RGB(212, 178, 236),
    Color::RGB(236, 174, 236), Color::RGB(236, 174, 192), Color::RGB(236, 180, 160), Color::RGB(228, 196, 144),
    Color::RGB(204, 210, 120), Color::RGB(180, 222, 120), Color::RGB(168, 230, 140), Color::RGB(168, 220, 184),
    Color::RGB(168, 204, 220), Color::RGB(160, 162, 160), Color::RGB(0, 0, 0), Color::RGB(0, 0, 0),
];


pub fn get_color_from_palette(palette_indx: u8) -> Color {
    NES_PALETTE[(palette_indx & 0x03F) as usize]
}

pub struct PPU {
    tbl_name: [[u8; 1024]; 2],
    tbl_palette:[u8; 32],

    vram_addr: u16,
    temp_addr: u16,
    fine_x: u8,
    write_toggle: bool,

    pub control: u8,
    mask: u8,
    pub status: u8,

    data_buffer: u8,

    scanline: i16,
    cycle: i16,

    pub emitted_nmi: bool,

    pub oam_addr: u8,
    pub oam_data: [u8; 256],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            tbl_name: [[0x00; 1024]; 2],
            tbl_palette: [0x00; 32],

            vram_addr: 0,
            temp_addr: 0,
            fine_x: 0,
            write_toggle: false,

            control: 0,
            mask: 0,
            status: 0,

            data_buffer: 0,

            scanline: 0,
            cycle: 0,

            emitted_nmi: false,

            oam_addr: 0,
            oam_data: [0x00; 256],
        }
    }

    pub fn step(&mut self) {
        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = -1;

                self.status &= !(1 << 7);
                self.status &= !(1 << 6);
            }
        }

        if self.scanline == 241 && self.cycle == 1 {
            self.status |=  1 << 7;

            if (self.control & 0x80) != 0 {
                self.emitted_nmi = true;
            }
        }

        let show_background = (self.mask & 0x08) != 0;
        let show_sprites = (self.mask & 0x10) != 0;

        if show_background && show_sprites {
            let sprite_0_y = self.oam_data[0] as i16;

            if self.scanline == sprite_0_y {
                if self.cycle == 2 {
                    self.status |= 1 << 6;
                }
            }
        }
    }

    pub fn cpu_read(&mut self, addr: u16, readonly: bool, rom: &mut Rom) -> u8 {
        let mut data: u8 = 0x00;

        match addr {
            0x0000 => {}, // Control (Write Only)
            0x0001 => {}, // Mask (Write Only)
            0x0002 => {   // Status
                data = (self.status & 0xE0) | (self.data_buffer & 0x1F);
                if !readonly {
                    self.status &= !(1 << 7);
                    self.write_toggle = false;
                }
            },
            0x0003 => {}, // OAM Addr
            0x0004 => { // OAM Data
               data = self.oam_data[self.oam_addr as usize];
            },

            0x0005 => {}, // Scroll
            0x0006 => {}, // PPU Addr
            0x0007 => {   // PPU Data
                data = self.data_buffer;
                self.data_buffer = self.ppu_read(self.vram_addr, rom);
                
                if self.vram_addr >= 0x3F00 {
                    data = self.data_buffer;
                }

                self.vram_addr = self.vram_addr.wrapping_add(if (self.control & 0x04) == 0 {1} else {32});
            },
            _ => {}
        }
        data
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8, rom: &mut Rom) {
        match addr {
            0x0000 => {
                let old_nmi = (self.control & 0x80) != 0;
                self.control = data;
                let new_nmi = (self.control & 0x80) != 0;

                self.temp_addr = (self.temp_addr & 0xF3FF) | ((data as u16 & 0x03) << 10);

                if (self.status & 0x80) != 0 && !old_nmi && new_nmi {
                    self.emitted_nmi = true;
                }
            },
            0x0001 => {
                self.mask = data;
            },

            0x0002 => {}, // Readonly
            0x0003 => {
                self.oam_addr = data;
            },

            0x0004 => {
                self.oam_data[self.oam_addr as usize] = data;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            },

            0x0005 => {
                if !self.write_toggle {
                    self.fine_x = data & 0x07;

                    self.temp_addr = (self.temp_addr & 0xFFE0) | ((data as u16) >> 3);
                    self.write_toggle = true;
                } else {
                    let fine_y = (data as u16 & 0x07) << 12;
                    let coarse_y = (data as u16 & 0xF8) << 2;

                    self.temp_addr = (self.temp_addr & 0x8FFF) | fine_y;
                    self.temp_addr = (self.temp_addr & 0xFC1F) | coarse_y;

                    self.write_toggle = false;
                }
            },
            0x0006 => {
                if !self.write_toggle {
                    self.temp_addr = (self.temp_addr & 0x00FF) | (((data as u16) & 0x3F) << 8);
                    self.write_toggle = true;
                } else {
                    self.temp_addr = (self.temp_addr & 0xFF00) | (data as u16);
                    self.vram_addr = self.temp_addr;
                    self.write_toggle = false;
                }
            },
            0x0007 => {
                self.ppu_write(self.vram_addr, data, rom);

                let increment = if (self.control & 0x04) == 0 { 1 } else { 32 };
                self.vram_addr = self.vram_addr.wrapping_add(increment);
            },
            _ => {}
        }
    }

    pub fn ppu_read(&self, addr: u16, rom: &Rom) -> u8 {
        let addr = addr & 0x3FFF;

        match addr {
            0x0000..=0x1FFF => {
                if (addr as usize) < rom.chr_rom.len() {
                    rom.chr_rom[addr as usize]
                } else {
                    0
                }
            },

            0x2000..=0x3EEF => {
                let masked_addr = addr & 0x0FFF;
                let vram_index = masked_addr & 0x03FF;
                let name_table = masked_addr / 0x0400;

                let final_idx = if rom.screen_mirroring {
                    if name_table == 0 || name_table == 2 { 0 } else { 1 }
                } else {
                    if name_table == 0 || name_table == 1 { 0 } else { 1 }
                };

                self.tbl_name[final_idx][vram_index as usize]

            },

            0x3F00..=0x3FFF => {
                let mut p_addr = addr & 0x001F;

                if p_addr == 0x0010 { p_addr = 0x0000; }
                if p_addr == 0x0014 { p_addr = 0x0004; }
                if p_addr == 0x0018 { p_addr = 0x0008; }
                if p_addr == 0x001C { p_addr = 0x000C; }

                self.tbl_palette[p_addr as usize]
            }

            _ => 0
        }
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8, rom: &mut Rom) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => {},

            0x2000..=0x3EFF => {
                let masked_addr = addr & 0x0FFF;
                let vram_index = masked_addr & 0x03FF;
                let name_table = masked_addr / 0x0400;

                let final_idx = if rom.screen_mirroring {
                    if name_table == 0 || name_table == 2 { 0 } else { 1 }
                    } else {
                    if name_table == 0 || name_table == 1 { 0 } else { 1 }
                };

                if final_idx < 0x0400 {
                    self.tbl_name[final_idx][vram_index as usize] = data;
                } else {
                    self.tbl_name[1][vram_index as usize] = data;
                }
            },

            0x3F00..=0x3FFF => {
                let mut p_addr = addr & 0x001F;
                if p_addr == 0x0010 { p_addr = 0x0000; }
                if p_addr == 0x0014 { p_addr = 0x0004; }
                if p_addr == 0x0018 { p_addr = 0x0008; }
                if p_addr == 0x001C { p_addr = 0x000C; }
                self.tbl_palette[p_addr as usize] = data;
            },

            _ => {}
        }

    }

    pub fn get_pattern_table(&self, rom: &Rom, table_idx: u8, palette_idx: u8) -> Vec<Color> {
        let base_addr: u16 = (table_idx as u16) << 12;

        let mut image_data = Vec::with_capacity(128 * 128);

        for tile_y in 0..16 {
            for tile_x in 0..16 {
                let tile_index = tile_y * 16 + tile_x;
                let tile_addr = base_addr + (tile_index as u16) * 16;

                for row in 0..8 {
                    let plane_0_byte = self.ppu_read(tile_addr + row, rom);
                    let plane_1_byte = self.ppu_read(tile_addr + row + 8, rom);

                    for col in 0..8 {
                        let pixel_bit_0 = (plane_0_byte >> (7 - col)) & 1;
                        let pixel_bit_1 = (plane_1_byte >> (7 - col)) & 1;

                        let color_index = (pixel_bit_1 << 1) | pixel_bit_0;

                        let palette_base = 0x3F00 + (palette_idx * 4) as u16;

                        let mut final_palette_index = self.ppu_read(palette_base + color_index as u16, rom);

                        if final_palette_index == 0 && color_index > 0 {
                            final_palette_index = 0x30;
                        }

                        let mut color: Color = get_color_from_palette(final_palette_index);
                        /*
                        color = match color_index {
                            0 => Color::RGB(0, 0, 0),       // Preto
                            1 => Color::RGB(255, 100, 100), // Cinza Escuro
                            2 => Color::RGB(170, 255, 170), // Cinza Claro
                            3 => Color::RGB(255, 255, 255), // Branco
                            _ => Color::RGB(0,0,0),
                        };
                        */
                        image_data.push(color);
                    }
                }
            }
        }
        image_data
    }
}
