use std::fs;

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];

struct Header {
    tag: [u8; 4],
    prg_rom_size: u8,
    chr_rom_size: u8,
    flags6: u8,
    flags7: u8,
}

pub struct Rom {
    pub header: Header,
    pub trainer: Vec<u8>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Rom {
    pub fn new(file_path: String) -> Self {
        
        const PRG_ROM_UNIT: usize = 16 * 1024;
        const CHR_ROM_UNIT: usize = 8 * 1024;
        const HEADER_SIZE: usize = 16;
        const TRAINER_SIZE: usize = 512;

        let file_data: Vec<u8> = fs::read(file_path).unwrap();
        if &file_data[0..4] == &NES_TAG {
            println!("The file is a valid ines format file!");
        } else {
            panic!("Cannot open the file!");
        }

        let header = Header {
            tag: file_data[0..4].try_into().unwrap(),
            prg_rom_size: file_data[4],
            chr_rom_size: file_data[5],
            flags6: file_data[6],
            flags7: file_data[7],
        };

        let trainer_present = header.flags6 & (1 << 2) != 0;
        let trainer_len = if trainer_present {TRAINER_SIZE} else {0};

        let trainer = file_data[HEADER_SIZE..(HEADER_SIZE + trainer_len)].to_vec();

        let prg_rom_len = header.prg_rom_size as usize * PRG_ROM_UNIT;
        let prg_rom_start = HEADER_SIZE + trainer_len;
        let prg_rom_end = prg_rom_start + prg_rom_len;

        let prg_rom = file_data[prg_rom_start..prg_rom_end].to_vec();

        let chr_rom_len = header.chr_rom_size as usize * CHR_ROM_UNIT;
        let chr_rom_start = prg_rom_end;
        let chr_rom_end = chr_rom_start + chr_rom_len;
        
        let chr_rom = file_data[chr_rom_start..chr_rom_end].to_vec();

        Rom {
            header: header,
            trainer: trainer,
            prg_rom: prg_rom,
            chr_rom: chr_rom,

        }
    }
}
