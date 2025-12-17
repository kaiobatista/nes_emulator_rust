
pub trait Mapper {
    fn cpu_read_mapper_addr(addr: u16) -> u16;
    
    fn cpu_write_mapper_addr(addr: u16) -> u16;
    
    fn ppu_read_mapper_addr(addr: u16) -> u16;
    
    fn ppu_write_mapper_addr(addr: u16) -> u16;
}

pub struct Mapper000 {
    pub npgr_banks: u8,
    pub nchr_banks: u8,
}

impl Mapper000 {
    pub fn new(npgr_banks: u8, nchr_banks: u8) -> Self {
        Mapper000 {
            npgr_banks: npgr_banks,
            nchr_banks: nchr_banks,
        }
    }
}

impl MapperTrait for Mapper000 {
    fn cpu_read_mapper_addr(&self, addr: u16) -> u16 {
        let offset = addr & 0x7FFF;

        if self.npgr_banks == 1 {
            offset & 0x3FFF
        } else {
            offset
        }
    }

    fn cpu_write_mapper_addr(&self, addr:u16) -> u16 {
        self.cpu_read_mapper_addr(addr)
    }
}
