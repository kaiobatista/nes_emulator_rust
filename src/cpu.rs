use crate::Bus;

pub struct Registers {
    pub a: u8,  // Acc
    pub x: u8,  // Reg X
    pub y: u8,  // Reg y
    pub pc: u16,// Program Counter
    pub sp: u8, // Stack Pointer
    pub f: u8,  // Flags
}

pub struct CPU<'lifetime> {
    pub cycles: usize,
    pub registers: Registers,
    pub bus: &'lifetime mut Bus,

    lookup_table: Vec<Instruction>,
    
    addr_abs: u16,
    addr_rel: u8,
    fetched_data: u8,
}

pub struct Instruction {
    pub name: &'static str,
    pub cycles: u8,
}

pub enum Flag {
    C = 1 << 0,
    Z = 1 << 1,
    I = 1 << 2,
    D = 1 << 3,
    B = 1 << 4,
    U = 1 << 5,
    V = 1 << 6,
    N = 1 << 7,
}

impl<'lifetime> CPU <'lifetime> {
    
    pub fn new(bus: &'lifetime mut Bus) -> Self {
        CPU {
            cycles: 0,
            registers: Registers {
                a: 0, x: 0, y: 0,
                pc: 0, sp: 0xFD,
                f: 0x24,
            },
            bus, 
            addr_abs: 0x0000,
            addr_rel: 0x00,
            fetched_data: 0x00,

            lookup_table: Instruction::lookup_table(),
        }
    }

    pub fn reset(&mut self) {
        self.registers.a = 0;
        self.registers.x = 0;
        self.registers.y = 0;
        self.registers.sp = 0xFD;
        self.registers.f = 0x24;
        
        self.addr_abs = 0xFFFC;
        self.addr_rel = 0x00;

        let low= self.bus.read(self.addr_abs);
        let high = self.bus.read(self.addr_abs.wrapping_add(1));

        self.registers.pc = (high as u16) << 8 | low as u16;

        self.addr_abs = 0x0000;
        self.fetched_data = 0x00;

        self.cycles += 8;
    }

    pub fn irq(&mut self) {
        if self.get_flag(Flag::I) == 0 {
            self.bus.write(0x0100 + self.registers.sp as u16, (((self.registers.pc as u16) >> 8) & 0x00FF) as u8);
            self.registers.sp = self.registers.sp.wrapping_sub(1);
            self.bus.write(0x0100 + self.registers.sp as u16, ((self.registers.pc as u16) & 0x00FF) as u8);
            self.registers.sp = self.registers.sp.wrapping_sub(1);

            self.set_flag(Flag::B, false);
            self.set_flag(Flag::U, true);
            self.set_flag(Flag::I, true);

            self.bus.write(0x0100 + self.registers.sp as u16, self.registers.f);
            self.registers.sp = self.registers.sp.wrapping_sub(1);

            self.addr_abs = 0xFFFE;
            let low = self.bus.read(self.addr_abs);
            let high = self.bus.read(self.addr_abs.wrapping_add(1));
            self.registers.pc = ((high as u16) << 8) | low as u16;

            self.cycles += 7;
        }
    }

    pub fn nmi(&mut self) {
        self.bus.write(0x0100 + self.registers.sp as u16, (((self.registers.pc as u16) >> 8) & 0x00FF) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus.write(0x0100 + self.registers.sp as u16, ((self.registers.pc as u16) & 0x00FF) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        self.set_flag(Flag::B, false);
        self.set_flag(Flag::U, true);
        self.set_flag(Flag::I, true);

        self.bus.write(0x0100 + self.registers.sp as u16, self.registers.f);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        self.addr_abs = 0xFFFA;
        let low = self.bus.read(self.addr_abs);
        let high = self.bus.read(self.addr_abs.wrapping_add(1));
        self.registers.pc = ((high as u16) << 8) | low as u16;

        self.cycles += 7;
    }

    pub fn step(&mut self) -> u8 {  // Return the cicles count
        let opcode = self.bus.read(self.registers.pc);
        // println!("PC: {:04X}", self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        match opcode {
            0x00 => {
                self.brk();
            }
           // ORA
            0x01 => { self.izx(); self.fetch(self.addr_abs); self.ora(); }
            0x0D => { self.abs(); self.fetch(self.addr_abs); self.ora(); }
            0x11 => { self.izy(); self.fetch(self.addr_abs); self.ora(); }
            0x15 => { self.zpx(); self.fetch(self.addr_abs); self.ora(); }
            0x19 => { self.aby(); self.fetch(self.addr_abs); self.ora(); }
            0x1D => { self.abx(); self.fetch(self.addr_abs); self.ora(); }

            // AND
            0x21 => { self.izx(); self.fetch(self.addr_abs); self.and(); }
            0x25 => { self.zp0(); self.fetch(self.addr_abs); self.and(); }
            0x2D => { self.abs(); self.fetch(self.addr_abs); self.and(); }
            0x31 => { self.izy(); self.fetch(self.addr_abs); self.and(); }
            0x35 => { self.zpx(); self.fetch(self.addr_abs); self.and(); }
            0x39 => { self.aby(); self.fetch(self.addr_abs); self.and(); }
            0x3D => { self.abx(); self.fetch(self.addr_abs); self.and(); }

            // EOR
            0x41 => { self.izx(); self.fetch(self.addr_abs); self.eor(); }
            0x45 => { self.zp0(); self.fetch(self.addr_abs); self.eor(); }
            0x4D => { self.abs(); self.fetch(self.addr_abs); self.eor(); }
            0x51 => { self.izy(); self.fetch(self.addr_abs); self.eor(); }
            0x55 => { self.zpx(); self.fetch(self.addr_abs); self.eor(); }
            0x59 => { self.aby(); self.fetch(self.addr_abs); self.eor(); }
            0x5D => { self.abx(); self.fetch(self.addr_abs); self.eor(); }

            // ADC
            0x61 => { self.izx(); self.fetch(self.addr_abs); self.adc(); }
            0x65 => { self.zp0(); self.fetch(self.addr_abs); self.adc(); }
            0x71 => { self.izy(); self.fetch(self.addr_abs); self.adc(); }
            0x75 => { self.zpx(); self.fetch(self.addr_abs); self.adc(); }
            0x79 => { self.aby(); self.fetch(self.addr_abs); self.adc(); }
            0x7D => { self.abx(); self.fetch(self.addr_abs); self.adc(); }

            // SBC
            0xE1 => { self.izx(); self.fetch(self.addr_abs); self.sbc(); }
            0xE5 => { self.zp0(); self.fetch(self.addr_abs); self.sbc(); }
            0xED => { self.abs(); self.fetch(self.addr_abs); self.sbc();}
            0xF1 => { self.izy(); self.fetch(self.addr_abs); self.sbc(); }
            0xF5 => { self.zpx(); self.fetch(self.addr_abs); self.sbc(); }
            0xF9 => { self.aby(); self.fetch(self.addr_abs); self.sbc(); }
            0xFD => { self.abx(); self.fetch(self.addr_abs); self.sbc(); }

            // CMP
            0xC1 => { self.izx(); self.fetch(self.addr_abs); self.cmp(); }
            0xC5 => { self.zp0(); self.fetch(self.addr_abs); self.cmp(); }
            0xCD => { self.abs(); self.fetch(self.addr_abs); self.cmp(); }
            0xD1 => { self.izy(); self.fetch(self.addr_abs); self.cmp(); }
            0xD5 => { self.zpx(); self.fetch(self.addr_abs); self.cmp(); }
            0xD9 => { self.aby(); self.fetch(self.addr_abs); self.cmp(); }
            0xDD => { self.abx(); self.fetch(self.addr_abs); self.cmp(); }

            // CPX
            0xE4 => { self.zp0(); self.fetch(self.addr_abs); self.cpx(); }
            0xEC => { self.abs(); self.fetch(self.addr_abs); self.cpx(); }

            // CPY
            0xC4 => { self.zp0(); self.fetch(self.addr_abs); self.cpy(); }
            0xCC => { self.abs(); self.fetch(self.addr_abs); self.cpy(); }

            // BIT (Falta o Absolute)
            0x2C => { self.abs(); self.fetch(self.addr_abs); self.bit(); }

            // --- LOADS & STORES (Modos Faltantes) ---

            // LDA
            0xA1 => { self.izx(); self.fetch(self.addr_abs); self.lda(); }
            0xA5 => { self.zp0(); self.fetch(self.addr_abs); self.lda(); }
            0xB1 => { self.izy(); self.fetch(self.addr_abs); self.lda(); }
            0xB5 => { self.zpx(); self.fetch(self.addr_abs); self.lda(); }
            0xB9 => { self.aby(); self.fetch(self.addr_abs); self.lda(); }
            0xBD => { self.abx(); self.fetch(self.addr_abs); self.lda(); }

            // LDX
            0xA6 => { self.zp0(); self.fetch(self.addr_abs); self.ldx(); }
            0xAE => { self.abs(); self.fetch(self.addr_abs); self.ldx(); }
            0xB6 => { self.zpy(); self.fetch(self.addr_abs); self.ldx(); }
            0xBE => { self.aby(); self.fetch(self.addr_abs); self.ldx(); }

            // LDY
            0xA4 => { self.zp0(); self.fetch(self.addr_abs); self.ldy(); }
            0xAC => { self.abs(); self.fetch(self.addr_abs); self.ldy(); }
            0xB4 => { self.zpx(); self.fetch(self.addr_abs); self.ldy(); }
            0xBC => { self.abx(); self.fetch(self.addr_abs); self.ldy(); }

            // STA (Store não usa fetch, apenas escreve)
            0x81 => { self.izx(); self.sta(); }
            0x8D => { self.abs(); self.sta(); }
            0x91 => { self.izy(); self.sta(); }
            0x95 => { self.zpx(); self.sta(); }
            0x99 => { self.aby(); self.sta(); }
            0x9D => { self.abx(); self.sta(); }

            // STX
            0x8E => { self.abs(); self.stx(); }
            0x96 => { self.zpy(); self.stx(); }

            // STY
            0x84 => { self.zp0(); self.sty(); }
            0x8C => { self.abs(); self.sty(); }
            0x94 => { self.zpx(); self.sty(); }

            // --- TRANSFERÊNCIAS (Registradores) ---
            0xAA => { self.imp(); self.tax(); }
            0xA8 => { self.imp(); self.tay(); }
            0x8A => { self.imp(); self.txa(); }
            0x98 => { self.imp(); self.tya(); }
            0x9A => { self.imp(); self.txs(); }
            0xBA => { self.imp(); self.tsx(); }

            // --- INCREMENTOS E DECREMENTOS ---
            
            // Increment Memory (Requer fetch antes!)
            0xE6 => { self.zp0(); self.fetch(self.addr_abs); self.inc(); }
            0xF6 => { self.zpx(); self.fetch(self.addr_abs); self.inc(); }
            0xEE => { self.abs(); self.fetch(self.addr_abs); self.inc(); }
            0xFE => { self.abx(); self.fetch(self.addr_abs); self.inc(); }

            // Decrement Memory (Requer fetch antes!)
            0xC6 => { self.zp0(); self.fetch(self.addr_abs); self.dec(); }
            0xD6 => { self.zpx(); self.fetch(self.addr_abs); self.dec(); }
            0xCE => { self.abs(); self.fetch(self.addr_abs); self.dec(); }
            0xDE => { self.abx(); self.fetch(self.addr_abs); self.dec(); }

            // Register Inc/Dec
            0xE8 => { self.imp(); self.inx(); }
            0xC8 => { self.imp(); self.iny(); }
            0xCA => { self.imp(); self.dex(); }
            // Nota: 0x88 (DEY) já estava implementado no seu código

            // --- JUMPS (Extra) ---
            0x6C => { self.ind(); self.jmp(); }

            // SHIFTS & ROTATES
            // ASL
            0x0A => { self.imp(); self.asl(true); }
            0x06 => { self.zp0(); self.fetch(self.addr_abs); self.asl(false); }
            0x16 => { self.zpx(); self.fetch(self.addr_abs); self.asl(false); }
            0x0E => { self.abs(); self.fetch(self.addr_abs); self.asl(false); }
            0x1E => { self.abx(); self.fetch(self.addr_abs); self.asl(false); }
            
            // LSR
            0x4A => { self.imp(); self.lsr(true); }
            0x46 => { self.zp0(); self.fetch(self.addr_abs); self.lsr(false); }
            0x56 => { self.zpx(); self.fetch(self.addr_abs); self.lsr(false); }
            0x4E => { self.abs(); self.fetch(self.addr_abs); self.lsr(false); }
            0x5E => { self.abx(); self.fetch(self.addr_abs); self.lsr(false); }

            // ROL
            0x2A => { self.imp(); self.rol(true); }
            0x26 => { self.zp0(); self.fetch(self.addr_abs); self.rol(false); }
            0x36 => { self.zpx(); self.fetch(self.addr_abs); self.rol(false); }
            0x2E => { self.abs(); self.fetch(self.addr_abs); self.rol(false); }
            0x3E => { self.abx(); self.fetch(self.addr_abs); self.rol(false); }

            // ROR
            0x6A => { self.imp(); self.ror(true); }
            0x66 => { self.zp0(); self.fetch(self.addr_abs); self.ror(false); }
            0x76 => { self.zpx(); self.fetch(self.addr_abs); self.ror(false); }
            0x6E => { self.abs(); self.fetch(self.addr_abs); self.ror(false); }
            0x7E => { self.abx(); self.fetch(self.addr_abs); self.ror(false); }

            0x05 => {
                self.zp0();
                self.fetch(self.addr_abs);
                self.ora();
            }

            0x08 => {
                self.imp();
                self.php();
            }

            0x09 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.ora();
            }

            0x10 => {
                self.rel();
                self.bpl();
            }

            0x18 => {
                self.imp();
                self.clc();
            }

            0x20 => {
                self.abs();
                self.jsr();
            }

            0x24 => {
                self.zp0();
                self.fetch(self.addr_abs);
                self.bit();
            }

            0x28 => {
                self.imp();
                self.plp();
            }

            0x29 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.and();
            }

            0x30 => {
                self.rel();
                self.bmi();
            }

            0x38 => {
                self.imp();
                self.sec();
            }

            0x40 => {
                self.imp();
                self.rti();
            }

            0x48 => {
                self.imp();
                self.pha();
            }

            0x49 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.eor();
            }

            0x4C => {
                self.abs();
                self.jmp();
            }

            0x50 => {
                self.rel();
                self.bvc();
            }

            0x60 => {
                self.imp();
                self.rts();
            }

            0x68 => {
                self.imp();
                self.pla();
            }

            0x70 => {
                self.rel();
                self.bvs();
            }

            0x78 => {
                self.imp();
                self.sei();
            }

            0x85 => {
                self.zp0();
                self.sta();
            }

            0x86 => {
                self.zp0();
                self.stx();
            }

            0x90 => {
                self.rel();
                self.bcc();
            }

            0xA0 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.ldy();
            }

            0xA2 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.ldx();
            }

            0xA9 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.lda();
            }

            0xAD => {
                self.abs();
                self.fetch(self.addr_abs);
                self.lda();
            }

            0xB0 => {
                self.rel();
                self.bcs();
            }

            0xB8 => {
                self.imp();
                self.clv();
            }

            0xC0 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.cpy();
            }

            0xC9 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.cmp();
            }

            0xD8 => {
                self.imp();
                self.cld();
            }

            0x69 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.adc();
            }

            0x6D => {
                self.abs();
                self.fetch(self.addr_abs);
                self.adc();
            }

            0x88 => {
                self.imp();
                self.dey();
            }

            0xD0 => {
                self.rel();
                self.bne();
            }

            0xE0 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.cpx();
            }

            0xE9 => {
                self.imm();
                self.fetch(self.addr_abs);
                self.sbc();
            }

            0xEA => {
                self.imp();
                self.nop();
            }

            0xF0 => {
                self.rel();
                self.beq();
            }

            0xF8 => {
                self.imp();
                self.sed();
            }

            _ => {
                panic!("Opcode not implemented: Ox{:02X}", opcode);
            }
        }

        self.cycles += self.lookup_table[opcode as usize].cycles as usize;
        // println!("OPCODE: {:02X} | NAME: {} | Fetched data: {:02X} | Absolute Addr: {:04X} | Relative Addr: {:02X}", opcode, self.lookup_table[opcode as usize].name, self.fetched_data, self.addr_abs, self.addr_rel);
        opcode
    }

    fn fetch(&mut self, addr: u16) -> u8 {
        self.fetched_data = self.bus.read(addr);
        self.fetched_data
    }

    pub fn get_flag(&self, f: Flag) -> u8 {
        if (self.registers.f & (f as u8)) != 0 {
            0x01
        } else {
            0x00
        }
    }

    pub fn set_flag(&mut self, f: Flag, state: bool) {
        if state {
            self.registers.f |= f as u8;
        } else {
            self.registers.f &= !(f as u8);
        }
    }

    fn imp(&mut self) -> (u16, u8) {
        (0, 0) 
    }

    fn imm(&mut self) -> (u16, u8) {
        self.addr_abs = self.registers.pc as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        (self.addr_abs, 0)
    }

    fn zp0(&mut self) -> (u16, u8) {
        self.addr_abs = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        (self.addr_abs, 0)
    }

    fn zpx(&mut self) -> (u16, u8) {
        self.addr_abs = self.bus.read(self.registers.pc).wrapping_add(self.registers.x) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        (self.addr_abs, 4)
    }

    fn zpy(&mut self) -> (u16, u8) {
        self.addr_abs = self.bus.read(self.registers.pc).wrapping_add(self.registers.y) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        self.addr_abs &= 0x00FF;
        (self.addr_abs, 4)
    }

    fn abs(&mut self) -> (u16, u8) {
        let low = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        let high = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);

        self.addr_abs = (high << 8) | low;
        
        (self.addr_abs, 0)
    }

    fn abx(&mut self) -> (u16, u8) {
        let low = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        let high = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);

        self.addr_abs = (high << 8) | low;
        self.addr_abs = self.addr_abs.wrapping_add(self.registers.x as u16) as u16;
        
        if (self.addr_abs & 0xFF00) != (high << 8) {
            return (self.addr_abs, 5);
        } else {
            return (self.addr_abs, 4);
        }
    }

    fn aby(&mut self) -> (u16, u8) {
        let low = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        let high = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);

        self.addr_abs = (high << 8) | low;
        self.addr_abs = self.addr_abs.wrapping_add(self.registers.y as u16) as u16;
        
        if (self.addr_abs & 0xFF00) != (high << 8) {
            return (self.addr_abs, 5);
        } else {
            return (self.addr_abs, 4);
        }
    }

    fn ind(&mut self) -> (u16, u8) {
        let ptr_low = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);
        let ptr_high = self.bus.read(self.registers.pc) as u16;
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let ptr = (ptr_high << 8) | ptr_low;

        if ptr_low == 0x00FF {
            self.addr_abs = (self.bus.read(ptr & 0xFF00) as u16) << 8 | self.bus.read(ptr) as u16;
        } else {
            self.addr_abs = ((self.bus.read((ptr + 1) as u16) as u16) << 8 | self.bus.read(ptr) as u16) as u16;
        }

        (self.addr_abs, 0)
    }

    fn izx(&mut self) -> (u16, u8) {
        let t = self.bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let low = self.bus.read(t.wrapping_add(self.registers.x) as u16) as u16;
        let high = self.bus.read(t.wrapping_add(self.registers.x).wrapping_add(1) as u16) as u16;

        self.addr_abs = (high << 8) | low;
        (self.addr_abs, 6)
    }

    fn izy(&mut self) -> (u16, u8) {
        let t = self.bus.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        let low = self.bus.read(t as u16) as u16;
        let high = self.bus.read(t.wrapping_add(1) as u16) as u16;

        self.addr_abs = (high << 8) | low;
        self.addr_abs = self.addr_abs.wrapping_add(self.registers.y as u16);

        if (self.addr_abs & 0xFF00) != (high << 8) {
            (self.addr_abs, 6)
        } else {
            (self.addr_abs, 5)
        }
    }

    fn rel(&mut self) -> (u16, u8) {
        self.addr_rel = self.bus.read(self.registers.pc);

        self.registers.pc = self.registers.pc.wrapping_add(1);
        (self.addr_rel as u16, 2)
    }

    // Operations
    
    fn adc(&mut self) -> u8 {
        let a = self.registers.a as u16;
        let m = self.fetched_data as u16;
        let c = self.get_flag(Flag::C) as u16;

        let result = a + m + c;
        self.registers.a = (result & 0x00FF) as u8;

        self.set_flag(Flag::C, result > 0xFF);
        self.set_flag(Flag::Z, (result & 0x00FF) == 0x00);
        self.set_flag(Flag::V, (!(a ^ m) & (a ^ result)) & 0x0080 != 0);
        self.set_flag(Flag::N, (result & 0x80) != 0x00);

        return 0
    }

    fn sbc(&mut self) -> u8 {
        let a = self.registers.a as u16;
        let m = (self.fetched_data as u16) ^ 0x00FF;
        let c = self.get_flag(Flag::C) as u16;

        let result = a + m + c;
        self.registers.a = (result & 0x00FF) as u8;

        self.set_flag(Flag::C, result > 0xFF);
        self.set_flag(Flag::Z, (result & 0x00FF) == 0);
        self.set_flag(Flag::V, ((a ^ result) & (m ^ result) & 0x0080) != 0);
        self.set_flag(Flag::N, (result & 0x80) != 0x00);

        return 0
    }

    fn and(&mut self) -> u8 {
        let result = self.registers.a & self.fetched_data;
        self.registers.a = result;

        self.set_flag(Flag::Z, result == 0x00);
        self.set_flag(Flag::N, (result & (1 << 7)) != 0x00);

        return 0
    }

    fn xxx(&mut self) -> u8 {
        return 0
    }

    fn brk(&mut self) -> u8 {
        self.registers.pc = self.registers.pc.wrapping_add(1);

        self.bus.write(0x0100 + self.registers.sp as u16, ((self.registers.pc >> 8) & 0xFF) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus.write(0x0100 + self.registers.sp as u16, (self.registers.pc & 0xFF) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        let status_to_stack = self.registers.f | (Flag::B as u8) | (Flag::U as u8);
        self.bus.write(0x0100 + self.registers.sp as u16, status_to_stack);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        self.set_flag(Flag::I, true);

        let low = self.bus.read(0xFFFE) as u16;
        let high = self.bus.read(0xFFFF) as u16;
        self.registers.pc = (high << 8) | low;

        0
    }

    fn ora(&mut self) -> u8 {
        self.registers.a |= self.fetched_data;

        self.set_flag(Flag::Z, self.registers.a == 0x00);
        self.set_flag(Flag::N, (self.registers.a & 0x80) != 0x00);

        return 0
    }

    fn eor(&mut self) -> u8 {
        let result = self.registers.a ^ self.fetched_data;
        self.registers.a = result;

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, (result & 0x80) != 0);

        return 0
    }

    fn lda(&mut self) -> u8 {
        self.registers.a = self.fetched_data;

        self.set_flag(Flag::Z, self.registers.a == 0x00);
        self.set_flag(Flag::N, (self.registers.a & 0x80) != 0x00);
        return 0
    }

    fn ldx(&mut self) -> u8 {
        self.registers.x = self.fetched_data;

        self.set_flag(Flag::Z, self.registers.x == 0x00);
        self.set_flag(Flag::N, (self.registers.x & 0x80) != 0x00);
        return 0
    }

    fn ldy(&mut self) -> u8 {
        self.registers.y = self.fetched_data;

        self.set_flag(Flag::Z, self.registers.y == 0x00);
        self.set_flag(Flag::N, (self.registers.y & 0x80) != 0x00);
        return 0
    }

    fn cmp(&mut self) -> u8 {
        self.set_flag(Flag::C, self.registers.a >= self.fetched_data);
        self.set_flag(Flag::Z, self.registers.a == self.fetched_data);
        self.set_flag(Flag::N, (self.registers.a.wrapping_sub(self.fetched_data) & 0x80) != 0);

        return 0;
    }

    fn cpx(&mut self) -> u8 {
        self.set_flag(Flag::C, self.registers.x >= self.fetched_data);
        self.set_flag(Flag::Z, self.registers.x == self.fetched_data);
        self.set_flag(Flag::N, (self.registers.x.wrapping_sub(self.fetched_data) & 0x80) != 0);

        return 0;
    }

    fn cpy(&mut self) -> u8 {
        self.set_flag(Flag::C, self.registers.y >= self.fetched_data);
        self.set_flag(Flag::Z, self.registers.y == self.fetched_data);
        self.set_flag(Flag::N, (self.registers.y.wrapping_sub(self.fetched_data) & 0x80) != 0);

        return 0;
    }

    fn sta(&mut self) -> u8 {
        self.bus.write(self.addr_abs, self.registers.a);

        return 0
    }

    fn stx(&mut self) -> u8 {
        self.bus.write(self.addr_abs, self.registers.x);

        return 0
    }

    fn sty(&mut self) -> u8 {
        self.bus.write(self.addr_abs, self.registers.y);

        return 0
    }

    fn txa(&mut self) -> u8 {
        self.registers.a = self.registers.x;
        self.set_flag(Flag::Z, self.registers.a == 0);
        self.set_flag(Flag::N, (self.registers.a & 0x80) != 0);

        return 0
    }

    fn tya(&mut self) -> u8 {
        self.registers.a = self.registers.y;
        self.set_flag(Flag::Z, self.registers.a == 0);
        self.set_flag(Flag::N, (self.registers.a & 0x80) != 0);

        return 0
    }

    fn tax(&mut self) -> u8 {
        self.registers.x = self.registers.a;
        self.set_flag(Flag::Z, self.registers.x == 0);
        self.set_flag(Flag::N, (self.registers.x & 0x80) != 0);

        return 0
    }

    fn tay(&mut self) -> u8 {
        self.registers.y = self.registers.a;
        self.set_flag(Flag::Z, self.registers.y == 0);
        self.set_flag(Flag::N, (self.registers.y & 0x80) != 0);

        return 0
    }

    fn tsx(&mut self) -> u8 {
        self.registers.x = self.registers.sp;
        self.set_flag(Flag::Z, self.registers.x == 0);
        self.set_flag(Flag::N, (self.registers.x & 0x80) != 0);

        return 0
    }

    fn txs(&mut self) -> u8 {
        self.registers.sp = self.registers.x;

        return 0
    }

    fn clc(&mut self) -> u8 {
        self.set_flag(Flag::C, false);
        return 0
    }

    fn cld(&mut self) -> u8 {
        self.set_flag(Flag::D, false);
        return 0
    }

    fn cli(&mut self) -> u8 {
        self.set_flag(Flag::I, false);
        return 0
    }

    fn clv(&mut self) -> u8 {
        self.set_flag(Flag::V, false);
        return 0
    }

    fn sec(&mut self) -> u8 {
        self.set_flag(Flag::C, true);
        return 0
    }

    fn sei(&mut self) -> u8 {
        self.set_flag(Flag::I, true);
        return 0
    }

    fn sed(&mut self) -> u8 {
        self.set_flag(Flag::D, true);
        return 0
    }

    fn nop(&mut self) -> u8 {
        return 0
    }

    fn rti(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let mut flags = self.bus.read(0x0100 + self.registers.sp as u16);

        flags |= Flag::U as u8;
        flags &= !(Flag::B as u8);

        self.registers.f = flags;

        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.registers.pc = self.bus.read(0x0100 + self.registers.sp as u16) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);
        self.registers.pc |= (self.bus.read(0x0100 + self.registers.sp as u16) as u16) << 8;

        return 0
        
    }

    fn rts(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let low = self.bus.read(0x0100 + self.registers.sp as u16) as u16;

        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = self.bus.read(0x0100 + self.registers.sp as u16) as u16;

        self.registers.pc = ((high << 8) | low);
        self.registers.pc = self.registers.pc + 1;

        return 0
    }

    fn inx(&mut self) -> u8 {
        self.registers.x = self.registers.x.wrapping_add(1);
        self.set_flag(Flag::Z, self.registers.x == 0);
        self.set_flag(Flag::N, (self.registers.x & 0x80) != 0);

        return 0
    }
    
    fn iny(&mut self) -> u8 {
        self.registers.y = self.registers.y.wrapping_add(1);
        self.set_flag(Flag::Z, self.registers.y == 0);
        self.set_flag(Flag::N, (self.registers.y & 0x80) != 0);

        return 0
    }

    fn inc(&mut self) -> u8 {
        let mut value = self.fetched_data;
        value = value.wrapping_add(1);
        self.bus.write(self.addr_abs, value);
    
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn dey(&mut self) -> u8 {
        self.registers.y = self.registers.y.wrapping_sub(1);
        self.set_flag(Flag::Z, self.registers.y == 0);
        self.set_flag(Flag::N, (self.registers.y & 0x80) != 0);

        return 0
    }

    fn dex(&mut self) -> u8 {
        self.registers.x = self.registers.x.wrapping_sub(1);
        self.set_flag(Flag::Z, self.registers.x == 0);
        self.set_flag(Flag::N, (self.registers.x & 0x80) != 0);

        return 0
    }

    fn dec(&mut self) -> u8 {
        let mut value = self.fetched_data;
        value = value.wrapping_sub(1);
        self.bus.write(self.addr_abs, value);
    
        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn asl(&mut self, is_mode_acc: bool) -> u8 {
        let mut value = if is_mode_acc {self.registers.a} else {self.fetched_data};
        
        self.set_flag(Flag::C, (value & 0x80) != 0);

        value = value << 1;

        if is_mode_acc {
            self.registers.a = value;
        } else {
            self.bus.write(self.addr_abs, value);
        }

        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn lsr(&mut self, is_mode_acc: bool) -> u8 {
        let mut value = if is_mode_acc {self.registers.a} else {self.fetched_data};
        
        self.set_flag(Flag::C, (value & 0x01) != 0);

        value = value >> 1;

        if is_mode_acc {
            self.registers.a = value;
        } else {
            self.bus.write(self.addr_abs, value);
        }

        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn rol(&mut self, is_mode_acc: bool) -> u8 {
        let mut value = if is_mode_acc {self.registers.a} else {self.fetched_data};
        
        let old_carry = self.get_flag(Flag::C);
        self.set_flag(Flag::C, (value & 0x80) != 0);

        value = (value << 1) | old_carry;

        if is_mode_acc {
            self.registers.a = value;
        } else {
            self.bus.write(self.addr_abs, value);
        }

        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn ror(&mut self, is_mode_acc: bool) -> u8 {
        let mut value = if is_mode_acc {self.registers.a} else {self.fetched_data};
        
        let old_carry = self.get_flag(Flag::C);
        self.set_flag(Flag::C, (value & 0x01) != 0);

        value = (value >> 1) | (old_carry << 7);

        if is_mode_acc {
            self.registers.a = value;
        } else {
            self.bus.write(self.addr_abs, value);
        }

        self.set_flag(Flag::Z, value == 0);
        self.set_flag(Flag::N, (value & 0x80) != 0);

        return 0
    }

    fn bcs(&mut self) -> u8 {
        if self.get_flag(Flag::C) == 1 {
            self.registers.pc = self.registers.pc.wrapping_add((self.addr_rel as i8 as i16) as u16);
        }
        return 0
    }

    fn bcc(&mut self) -> u8 {
        if self.get_flag(Flag::C) == 0 {
            self.registers.pc = self.registers.pc.wrapping_add((self.addr_rel as i8 as i16) as u16);
        }
        return 0
    }

    fn bvs(&mut self) -> u8 {
        if self.get_flag(Flag::V) == 1 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
        }
        return 0
    }

    fn bvc(&mut self) -> u8 {
        if self.get_flag(Flag::V) == 0 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
        }
        return 0
    }
    
    fn beq(&mut self) -> u8 {
        if self.get_flag(Flag::Z) == 1 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
            return 1
        }
        return 0
    }

    fn bne(&mut self) -> u8 {
        if self.get_flag(Flag::Z) != 1 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
            return 1
        }
        return 0
    }

    fn bpl(&mut self) -> u8 {
        if self.get_flag(Flag::N) == 0 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
            return 1
        }
        return 0
    }

    fn bmi(&mut self) -> u8 {
        if self.get_flag(Flag::N) != 0 {
            self.registers.pc = self.registers.pc.wrapping_add(self.addr_rel as i8 as i16 as u16);
            return 1
        }
        return 0
    }

    fn jsr(&mut self) -> u8 {
        self.registers.pc = self.registers.pc.wrapping_sub(1);

        self.bus.write(0x0100 | self.registers.sp as u16, ((self.registers.pc & 0xFF00) >> 8) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.bus.write(0x0100 | self.registers.sp as u16, (self.registers.pc & 0x00FF) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        self.registers.pc = self.addr_abs;

        return 0
    }

    fn jmp(&mut self) -> u8 {
        self.registers.pc = self.addr_abs;

        return 0
    }
    
    fn bit(&mut self) -> u8 {
        let result = self.registers.a & self.fetched_data;

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::V, (self.fetched_data & (1 << 6)) != 0);
        self.set_flag(Flag::N, (self.fetched_data & (1 << 7)) != 0);

        return 0
    }

    fn php(&mut self) -> u8 {
        self.bus.write(0x0100 + self.registers.sp as u16, self.registers.f | (Flag::B as u8) | (Flag::U as u8));
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        return 0
    }

    fn pha(&mut self) -> u8 {
        self.bus.write(0x0100 + self.registers.sp as u16, self.registers.a);
        self.registers.sp = self.registers.sp.wrapping_sub(1);

        return 0
    }

    fn pla(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let result = self.bus.read(0x0100 + self.registers.sp as u16);

        self.registers.a = result;

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, (result & (1 << 7)) != 0);

        return 0
    }

    fn plp(&mut self) -> u8 {
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let mut result = self.bus.read(0x0100 + self.registers.sp as u16);

        result |= Flag::U as u8;
        result &= !(Flag::B as u8);

        self.registers.f = result;

        return 0
    }
}

impl Instruction {
    pub fn lookup_table() -> Vec<Instruction> {
        vec![
// 0x00: BRK Implied
Instruction {
    name: "BRK",
    cycles: 7,
},
// 0x01: ORA Indirect, X
Instruction {
    name: "ORA",
    cycles: 6,
},
// 0x02: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x03: XXX Indirect, X (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x04: XXX Zero Page (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 3,
},
// 0x05: ORA Zero Page
Instruction {
    name: "ORA",
    cycles: 3,
},
// 0x06: ASL Zero Page
Instruction {
    name: "ASL",
    cycles: 5,
},
// 0x07: XXX Zero Page (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x08: PHP Implied
Instruction {
    name: "PHP",
    cycles: 3,
},
// 0x09: ORA Immediate
Instruction {
    name: "ORA",
    cycles: 2,
},
// 0x0A: ASL Accumulator
Instruction {
    name: "ASL",
    cycles: 2,
},
// 0x0B: XXX Immediate (ANC Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x0C: XXX Absolute (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x0D: ORA Absolute
Instruction {
    name: "ORA",
    cycles: 4,
},
// 0x0E: ASL Absolute
Instruction {
    name: "ASL",
    cycles: 6,
},
// 0x0F: XXX Absolute (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x10: BPL Relative
Instruction {
    name: "BPL",
    cycles: 2,
},
// 0x11: ORA Indirect, Y
Instruction {
    name: "ORA",
    cycles: 5,
},
// 0x12: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x13: XXX Indirect, Y (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x14: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x15: ORA Zero Page, X
Instruction {
    name: "ORA",
    cycles: 4,
},
// 0x16: ASL Zero Page, X
Instruction {
    name: "ASL",
    cycles: 6,
},
// 0x17: XXX Zero Page, X (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x18: CLC Implied
Instruction {
    name: "CLC",
    cycles: 2,
},
// 0x19: ORA Absolute, Y
Instruction {
    name: "ORA",
    cycles: 4,
},
// 0x1A: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x1B: XXX Absolute, Y (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x1C: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x1D: ORA Absolute, X
Instruction {
    name: "ORA",
    cycles: 4,
},
// 0x1E: ASL Absolute, X
Instruction {
    name: "ASL",
    cycles: 7,
},
// 0x1F: XXX Absolute, X (SLO Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x20: JSR Absolute
Instruction {
    name: "JSR",
    cycles: 6,
},
// 0x21: AND Indirect, X
Instruction {
    name: "AND",
    cycles: 6,
},
// 0x22: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x23: XXX Indirect, X (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x24: BIT Zero Page
Instruction {
    name: "BIT",
    cycles: 3,
},
// 0x25: AND Zero Page
Instruction {
    name: "AND",
    cycles: 3,
},
// 0x26: ROL Zero Page
Instruction {
    name: "ROL",
    cycles: 5,
},
// 0x27: XXX Zero Page (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x28: PLP Implied
Instruction {
    name: "PLP",
    cycles: 4,
},
// 0x29: AND Immediate
Instruction {
    name: "AND",
    cycles: 2,
},
// 0x2A: ROL Accumulator
Instruction {
    name: "ROL",
    cycles: 2,
},
// 0x2B: XXX Immediate (ANC Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x2C: BIT Absolute
Instruction {
    name: "BIT",
    cycles: 4,
},
// 0x2D: AND Absolute
Instruction {
    name: "AND",
    cycles: 4,
},
// 0x2E: ROL Absolute
Instruction {
    name: "ROL",
    cycles: 6,
},
// 0x2F: XXX Absolute (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x30: BMI Relative
Instruction {
    name: "BMI",
    cycles: 2,
},
// 0x31: AND Indirect, Y
Instruction {
    name: "AND",
    cycles: 5,
},
// 0x32: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x33: XXX Indirect, Y (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x34: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x35: AND Zero Page, X
Instruction {
    name: "AND",
    cycles: 4,
},
// 0x36: ROL Zero Page, X
Instruction {
    name: "ROL",
    cycles: 6,
},
// 0x37: XXX Zero Page, X (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x38: SEC Implied
Instruction {
    name: "SEC",
    cycles: 2,
},
// 0x39: AND Absolute, Y
Instruction {
    name: "AND",
    cycles: 4,
},
// 0x3A: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x3B: XXX Absolute, Y (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x3C: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x3D: AND Absolute, X
Instruction {
    name: "AND",
    cycles: 4,
},
// 0x3E: ROL Absolute, X
Instruction {
    name: "ROL",
    cycles: 7,
},
// 0x3F: XXX Absolute, X (RLA Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x40: RTI Implied
Instruction {
    name: "RTI",
    cycles: 6,
},
// 0x41: EOR Indirect, X
Instruction {
    name: "EOR",
    cycles: 6,
},
// 0x42: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x43: XXX Indirect, X (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x44: XXX Zero Page (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 3,
},
// 0x45: EOR Zero Page
Instruction {
    name: "EOR",
    cycles: 3,
},
// 0x46: LSR Zero Page
Instruction {
    name: "LSR",
    cycles: 5,
},
// 0x47: XXX Zero Page (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x48: PHA Implied
Instruction {
    name: "PHA",
    cycles: 3,
},
// 0x49: EOR Immediate
Instruction {
    name: "EOR",
    cycles: 2,
},
// 0x4A: LSR Accumulator
Instruction {
    name: "LSR",
    cycles: 2,
},
// 0x4B: XXX Immediate (ALR Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x4C: JMP Absolute
Instruction {
    name: "JMP",
    cycles: 3,
},
// 0x4D: EOR Absolute
Instruction {
    name: "EOR",
    cycles: 4,
},
// 0x4E: LSR Absolute
Instruction {
    name: "LSR",
    cycles: 6,
},
// 0x4F: XXX Absolute (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x50: BVC Relative
Instruction {
    name: "BVC",
    cycles: 2,
},
// 0x51: EOR Indirect, Y
Instruction {
    name: "EOR",
    cycles: 5,
},
// 0x52: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x53: XXX Indirect, Y (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x54: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x55: EOR Zero Page, X
Instruction {
    name: "EOR",
    cycles: 4,
},
// 0x56: LSR Zero Page, X
Instruction {
    name: "LSR",
    cycles: 6,
},
// 0x57: XXX Zero Page, X (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x58: CLI Implied
Instruction {
    name: "CLI",
    cycles: 2,
},
// 0x59: EOR Absolute, Y
Instruction {
    name: "EOR",
    cycles: 4,
},
// 0x5A: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x5B: XXX Absolute, Y (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x5C: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x5D: EOR Absolute, X
Instruction {
    name: "EOR",
    cycles: 4,
},
// 0x5E: LSR Absolute, X
Instruction {
    name: "LSR",
    cycles: 7,
},
// 0x5F: XXX Absolute, X (SRE Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x60: RTS Implied
Instruction {
    name: "RTS",
    cycles: 6,
},
// 0x61: ADC Indirect, X
Instruction {
    name: "ADC",
    cycles: 6,
},
// 0x62: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x63: XXX Indirect, X (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x64: XXX Zero Page (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 3,
},
// 0x65: ADC Zero Page
Instruction {
    name: "ADC",
    cycles: 3,
},
// 0x66: ROR Zero Page
Instruction {
    name: "ROR",
    cycles: 5,
},
// 0x67: XXX Zero Page (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x68: PLA Implied
Instruction {
    name: "PLA",
    cycles: 4,
},
// 0x69: ADC Immediate
Instruction {
    name: "ADC",
    cycles: 2,
},
// 0x6A: ROR Accumulator
Instruction {
    name: "ROR",
    cycles: 2,
},
// 0x6B: XXX Immediate (ARR Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x6C: JMP Indirect
Instruction {
    name: "JMP",
    cycles: 5,
},
// 0x6D: ADC Absolute
Instruction {
    name: "ADC",
    cycles: 4,
},
// 0x6E: ROR Absolute
Instruction {
    name: "ROR",
    cycles: 6,
},
// 0x6F: XXX Absolute (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x70: BVS Relative
Instruction {
    name: "BVS",
    cycles: 2,
},
// 0x71: ADC Indirect, Y
Instruction {
    name: "ADC",
    cycles: 5,
},
// 0x72: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x73: XXX Indirect, Y (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0x74: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x75: ADC Zero Page, X
Instruction {
    name: "ADC",
    cycles: 4,
},
// 0x76: ROR Zero Page, X
Instruction {
    name: "ROR",
    cycles: 6,
},
// 0x77: XXX Zero Page, X (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x78: SEI Implied
Instruction {
    name: "SEI",
    cycles: 2,
},
// 0x79: ADC Absolute, Y
Instruction {
    name: "ADC",
    cycles: 4,
},
// 0x7A: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x7B: XXX Absolute, Y (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x7C: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x7D: ADC Absolute, X
Instruction {
    name: "ADC",
    cycles: 4,
},
// 0x7E: ROR Absolute, X
Instruction {
    name: "ROR",
    cycles: 7,
},
// 0x7F: XXX Absolute, X (RRA Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0x80: XXX Immediate (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x81: STA Indirect, X
Instruction {
    name: "STA",
    cycles: 6,
},
// 0x82: XXX Immediate (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x83: XXX Indirect, X (SAX Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x84: STY Zero Page
Instruction {
    name: "STY",
    cycles: 3,
},
// 0x85: STA Zero Page
Instruction {
    name: "STA",
    cycles: 3,
},
// 0x86: STX Zero Page
Instruction {
    name: "STX",
    cycles: 3,
},
// 0x87: XXX Zero Page (SAX Illegal)
Instruction {
    name: "XXX",
    cycles: 3,
},
// 0x88: DEY Implied
Instruction {
    name: "DEY",
    cycles: 2,
},
// 0x89: XXX Immediate (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x8A: TXA Implied
Instruction {
    name: "TXA",
    cycles: 2,
},
// 0x8B: XXX Immediate (XAA Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x8C: STY Absolute
Instruction {
    name: "STY",
    cycles: 4,
},
// 0x8D: STA Absolute
Instruction {
    name: "STA",
    cycles: 4,
},
// 0x8E: STX Absolute
Instruction {
    name: "STX",
    cycles: 4,
},
// 0x8F: XXX Absolute (SAX Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x90: BCC Relative
Instruction {
    name: "BCC",
    cycles: 2,
},
// 0x91: STA Indirect, Y
Instruction {
    name: "STA",
    cycles: 6,
},
// 0x92: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0x93: XXX Indirect, Y (AHX Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0x94: STY Zero Page, X
Instruction {
    name: "STY",
    cycles: 4,
},
// 0x95: STA Zero Page, X
Instruction {
    name: "STA",
    cycles: 4,
},
// 0x96: STX Zero Page, Y
Instruction {
    name: "STX",
    cycles: 4,
},
// 0x97: XXX Zero Page, Y (SAX Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0x98: TYA Implied
Instruction {
    name: "TYA",
    cycles: 2,
},
// 0x99: STA Absolute, Y
Instruction {
    name: "STA",
    cycles: 5,
},
// 0x9A: TXS Implied
Instruction {
    name: "TXS",
    cycles: 2,
},
// 0x9B: XXX Absolute, Y (TAS Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x9C: XXX Absolute, X (SHY Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x9D: STA Absolute, X
Instruction {
    name: "STA",
    cycles: 5,
},
// 0x9E: XXX Absolute, Y (SHX Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0x9F: XXX Absolute, Y (AHX Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0xA0: LDY Immediate
Instruction {
    name: "LDY",
    cycles: 2,
},
// 0xA1: LDA Indirect, X
Instruction {
    name: "LDA",
    cycles: 6,
},
// 0xA2: LDX Immediate
Instruction {
    name: "LDX",
    cycles: 2,
},
// 0xA3: XXX Indirect, X (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0xA4: LDY Zero Page
Instruction {
    name: "LDY",
    cycles: 3,
},
// 0xA5: LDA Zero Page
Instruction {
    name: "LDA",
    cycles: 3,
},
// 0xA6: LDX Zero Page
Instruction {
    name: "LDX",
    cycles: 3,
},
// 0xA7: XXX Zero Page (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 3,
},
// 0xA8: TAY Implied
Instruction {
    name: "TAY",
    cycles: 2,
},
// 0xA9: LDA Immediate
Instruction {
    name: "LDA",
    cycles: 2,
},
// 0xAA: TAX Implied
Instruction {
    name: "TAX",
    cycles: 2,
},
// 0xAB: XXX Immediate (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xAC: LDY Absolute
Instruction {
    name: "LDY",
    cycles: 4,
},
// 0xAD: LDA Absolute
Instruction {
    name: "LDA",
    cycles: 4,
},
// 0xAE: LDX Absolute
Instruction {
    name: "LDX",
    cycles: 4,
},
// 0xAF: XXX Absolute (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xB0: BCS Relative
Instruction {
    name: "BCS",
    cycles: 2,
},
// 0xB1: LDA Indirect, Y
Instruction {
    name: "LDA",
    cycles: 5,
},
// 0xB2: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xB3: XXX Indirect, Y (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0xB4: LDY Zero Page, X
Instruction {
    name: "LDY",
    cycles: 4,
},
// 0xB5: LDA Zero Page, X
Instruction {
    name: "LDA",
    cycles: 4,
},
// 0xB6: LDX Zero Page, Y
Instruction {
    name: "LDX",
    cycles: 4,
},
// 0xB7: XXX Zero Page, Y (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xB8: CLV Implied
Instruction {
    name: "CLV",
    cycles: 2,
},
// 0xB9: LDA Absolute, Y
Instruction {
    name: "LDA",
    cycles: 4,
},
// 0xBA: TSX Implied
Instruction {
    name: "TSX",
    cycles: 2,
},
// 0xBB: XXX Absolute, Y (LAS Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xBC: LDY Absolute, X
Instruction {
    name: "LDY",
    cycles: 4,
},
// 0xBD: LDA Absolute, X
Instruction {
    name: "LDA",
    cycles: 4,
},
// 0xBE: LDX Absolute, Y
Instruction {
    name: "LDX",
    cycles: 4,
},
// 0xBF: XXX Absolute, Y (LAX Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xC0: CPY Immediate
Instruction {
    name: "CPY",
    cycles: 2,
},
// 0xC1: CMP Indirect, X
Instruction {
    name: "CMP",
    cycles: 6,
},
// 0xC2: XXX Immediate (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xC3: XXX Indirect, X (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0xC4: CPY Zero Page
Instruction {
    name: "CPY",
    cycles: 3,
},
// 0xC5: CMP Zero Page
Instruction {
    name: "CMP",
    cycles: 3,
},
// 0xC6: DEC Zero Page
Instruction {
    name: "DEC",
    cycles: 5,
},
// 0xC7: XXX Zero Page (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0xC8: INY Implied
Instruction {
    name: "INY",
    cycles: 2,
},
// 0xC9: CMP Immediate
Instruction {
    name: "CMP",
    cycles: 2,
},
// 0xCA: DEX Implied
Instruction {
    name: "DEX",
    cycles: 2,
},
// 0xCB: XXX Immediate (AXS Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xCC: CPY Absolute
Instruction {
    name: "CPY",
    cycles: 4,
},
// 0xCD: CMP Absolute
Instruction {
    name: "CMP",
    cycles: 4,
},
// 0xCE: DEC Absolute
Instruction {
    name: "DEC",
    cycles: 6,
},
// 0xCF: XXX Absolute (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0xD0: BNE Relative
Instruction {
    name: "BNE",
    cycles: 2,
},
// 0xD1: CMP Indirect, Y
Instruction {
    name: "CMP",
    cycles: 5,
},
// 0xD2: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xD3: XXX Indirect, Y (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0xD4: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xD5: CMP Zero Page, X
Instruction {
    name: "CMP",
    cycles: 4,
},
// 0xD6: DEC Zero Page, X
Instruction {
    name: "DEC",
    cycles: 6,
},
// 0xD7: XXX Zero Page, X (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0xD8: CLD Implied
Instruction {
    name: "CLD",
    cycles: 2,
},
// 0xD9: CMP Absolute, Y
Instruction {
    name: "CMP",
    cycles: 4,
},
// 0xDA: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xDB: XXX Absolute, Y (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0xDC: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xDD: CMP Absolute, X
Instruction {
    name: "CMP",
    cycles: 4,
},
// 0xDE: DEC Absolute, X
Instruction {
    name: "DEC",
    cycles: 7,
},
// 0xDF: XXX Absolute, X (DCP Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0xE0: CPX Immediate
Instruction {
    name: "CPX",
    cycles: 2,
},
// 0xE1: SBC Indirect, X
Instruction {
    name: "SBC",
    cycles: 6,
},
// 0xE2: XXX Immediate (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xE3: XXX Indirect, X (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0xE4: CPX Zero Page
Instruction {
    name: "CPX",
    cycles: 3,
},
// 0xE5: SBC Zero Page
Instruction {
    name: "SBC",
    cycles: 3,
},
// 0xE6: INC Zero Page
Instruction {
    name: "INC",
    cycles: 5,
},
// 0xE7: XXX Zero Page (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 5,
},
// 0xE8: INX Implied
Instruction {
    name: "INX",
    cycles: 2,
},
// 0xE9: SBC Immediate
Instruction {
    name: "SBC",
    cycles: 2,
},
// 0xEA: NOP Implied
Instruction {
    name: "NOP",
    cycles: 2,
},
// 0xEB: XXX Immediate (SBC Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xEC: CPX Absolute
Instruction {
    name: "CPX",
    cycles: 4,
},
// 0xED: SBC Absolute
Instruction {
    name: "SBC",
    cycles: 4,
},
// 0xEE: INC Absolute
Instruction {
    name: "INC",
    cycles: 6,
},
// 0xEF: XXX Absolute (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0xF0: BEQ Relative
Instruction {
    name: "BEQ",
    cycles: 2,
},
// 0xF1: SBC Indirect, Y
Instruction {
    name: "SBC",
    cycles: 5,
},
// 0xF2: XXX Implied (Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xF3: XXX Indirect, Y (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 8,
},
// 0xF4: XXX Zero Page, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xF5: SBC Zero Page, X
Instruction {
    name: "SBC",
    cycles: 4,
},
// 0xF6: INC Zero Page, X
Instruction {
    name: "INC",
    cycles: 6,
},
// 0xF7: XXX Zero Page, X (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 6,
},
// 0xF8: SED Implied
Instruction {
    name: "SED",
    cycles: 2,
},
// 0xF9: SBC Absolute, Y
Instruction {
    name: "SBC",
    cycles: 4,
},
// 0xFA: XXX Implied (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 2,
},
// 0xFB: XXX Absolute, Y (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},
// 0xFC: XXX Absolute, X (NOP Illegal)
Instruction {
    name: "XXX",
    cycles: 4,
},
// 0xFD: SBC Absolute, X
Instruction {
    name: "SBC",
    cycles: 4,
},
// 0xFE: INC Absolute, X
Instruction {
    name: "INC",
    cycles: 7,
},
// 0xFF: XXX Absolute, X (ISC Illegal)
Instruction {
    name: "XXX",
    cycles: 7,
},            
        ]
    }
}

