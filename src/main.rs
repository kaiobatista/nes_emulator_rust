mod bus;
mod cpu;
mod ppu;
mod sdl_ui;
mod ines_file;
mod controller;

use bus::Bus;
use cpu::CPU;
use ines_file::Rom;
use ppu::PPU;
use controller::Controller;

fn main() {
    
    let rom = Rom::new("nestest.nes".to_string());
    let ppu: PPU = PPU::new();

    let mut bus = Bus {
        ram: [0x00; 2 * 1024],
        rom: rom,
        ppu: ppu,
        controller: [Controller::new(), Controller::new()],
    };

    let mut cpu = CPU::new(&mut bus);
    
    /*
    cpu.bus.write(0x8000, 0xA9); // LDA
    cpu.bus.write(0x8001, 0x05);
    cpu.bus.write(0x8002, 0xA0); // LDY
    cpu.bus.write(0x8003, 0x08);
    cpu.bus.write(0x8004, 0x88); // DEY
    cpu.bus.write(0x8005, 0x6D); // ADC
    cpu.bus.write(0x8006, 0x01);
    cpu.bus.write(0x8007, 0x80);
    cpu.bus.write(0x8008, 0x88); // DEY
    cpu.bus.write(0x8009, 0xD0); // BNE
    cpu.bus.write(0x800A, 0xFA);
    */
    cpu.reset();

    println!("PC: ${:04X}", cpu.registers.pc);
    
    sdl_ui::start_ui(cpu);

    println!("Exit with success!");

/*
    for _ in 0..10 {
        println!("${:04X}: #{:02X} | Cycle: {}", cpu.registers.pc, cpu.bus.read(cpu.registers.pc), cpu.cycles);
        println!("A: {:02X} | X: {:02X} | Y: {:02X} |", cpu.registers.a, cpu.registers.x, cpu.registers.y);
        cpu.step();
    }
*/
}
