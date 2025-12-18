use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;

use std::time::{Duration, Instant};
use std::thread;

use crate::cpu::CPU;
use crate::ppu::{PPU, get_color_from_palette};
use crate::ines_file::Rom;
use crate::controller::Button;


pub fn start_ui(mut cpu: CPU) {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES Emulator Debugger", 256, 240)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();

    let texture_creator = canvas.texture_creator();

    let font_path = "/usr/share/fonts/TTF/FiraCode-Medium.ttf";
    let mut font = ttf_context.load_font(font_path, 20).unwrap();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let target_duration = Duration::from_micros(16_667);

    'running: loop {
        let frame_start = Instant::now();
        
        if !handle_input(&mut event_pump, &mut cpu) {
            break 'running;
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let mut cycles_this_frame = 0;
        let mut mid_frame_check = false;
        while cycles_this_frame < 29780 {
            // 1. Executa 1 instrução da CPU
            let cycles = cpu.step() as usize; 
            cycles_this_frame += cycles;

            // 2. A PPU roda 3 vezes para cada 1 ciclo de CPU
            for _ in 0..(cycles * 3) {
                 cpu.bus.ppu.step();
            }
            
            if cycles_this_frame > 15000 && !mid_frame_check {
                 if !handle_input(&mut event_pump, &mut cpu) {
                    break 'running;
                }
                mid_frame_check = true;
            }
            

            if cpu.bus.ppu.emitted_nmi {
                cpu.nmi();
                cpu.bus.ppu.emitted_nmi = false;
            }

        }
        
        //render_debug_info(&mut canvas, &mut font, &texture_creator, &mut cpu);

        // render_pattern_table(&mut canvas, &texture_creator, &cpu.bus.ppu, &cpu.bus.rom, 0, 850, 300);
        // render_pattern_table(&mut canvas, &texture_creator, &cpu.bus.ppu, &cpu.bus.rom, 1, 850, 600);
        
        render_nametable(&mut canvas, &texture_creator, &cpu.bus.ppu, &cpu.bus.rom);

        let elapsed = frame_start.elapsed();
        if elapsed < target_duration {
            thread::sleep(target_duration - elapsed);
        }

        canvas.present();
    }
}

fn handle_input(event_pump: &mut sdl2::EventPump, cpu: &mut CPU) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} => {
                return false
            }

            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                cpu.step();
                for _ in 0..3 {
                    cpu.bus.ppu.step();
                }
            }

            Event::KeyDown {
                keycode: Some(Keycode::R),
                ..
            } => {
                cpu.reset();
                println!("Reset!");
            }

            Event::KeyDown {
                keycode: Some(Keycode::N),
                ..
            } => {
                cpu.nmi();
                println!("NMI!")
            }

            Event::KeyDown { keycode: Some(key), repeat, ..} => {
                match key  {
                    Keycode::Up => cpu.bus.controller[0].set_button(Button::UP, true),
                    Keycode::Down => cpu.bus.controller[0].set_button(Button::DOWN, true),
                    Keycode::KpEnter => cpu.bus.controller[0].set_button(Button::START, true),
                    _ => {}
                }
            }

            Event::KeyUp { keycode: Some(key), repeat, ..} => {
                match key  {
                    Keycode::Up => cpu.bus.controller[0].set_button(Button::UP, false),
                    Keycode::Down => cpu.bus.controller[0].set_button(Button::DOWN, false),
                    Keycode::KpEnter => cpu.bus.controller[0].set_button(Button::START, false),
                    _ => {}
                }
            }

            _ => {}
        }
    }
    return true
}

fn draw_text<'a>(text: String, color: Color, position: Vec<i32>, canvas: &mut Canvas<Window>, font: &mut Font, texture_creator: &'a TextureCreator<sdl2::video::WindowContext>) {
    let surface = font.render(&text)
        .blended(color)
        .unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();
    let target_rect = Rect::new(
        position[0].into(),
        position[1].into(),
        texture.query().width,
        texture.query().height,
    );
    canvas.copy(&texture, None, Some(target_rect)).unwrap();
}

fn render_debug_info<'a>(
    canvas: &mut Canvas<Window>,
    font: &mut Font,
    texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    cpu: &mut CPU,
    ) {
        let font_height = font.height();

        for i in 0x00..0x0F + 1 {
            let mut text = format!("${:04X}: ", (i as u16) << 4);
            for j in 0x00..0x0F + 1 {
                text.push_str(&format!("{:02X} ", cpu.bus.read((i as u16) << 4 | j as u16)));
            }
            let surface = font.render(&text)
            .blended(Color::RGB(255, 255, 255))
            .unwrap();

            let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

            let target_rect = Rect::new(
                10,
                10 + i * texture.query().height as i32,
                texture.query().width,
                texture.query().height,
            );
            canvas.copy(&texture, None, Some(target_rect)).unwrap();
        } 
        
        for i in 0x00..0x0F + 1 {
            let mut text = format!("${:04X}: ", 0x8000 | (i as u16) << 4);
            for j in 0x00..0x0F + 1 {
                text.push_str(&format!("{:02X} ", cpu.bus.read(0x8000 | (i as u16) << 4 | j as u16)));
            }
            let surface = font.render(&text)
            .blended(Color::RGB(255, 255, 255))
            .unwrap();

            let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

            let target_rect = Rect::new(
                10,
                420 + i * texture.query().height as i32,
                texture.query().width,
                texture.query().height,
            );
            canvas.copy(&texture, None, Some(target_rect)).unwrap();
        }

        draw_text(format!("Status: 0b{:08b} [0x{:02X}]", cpu.registers.f, cpu.registers.f), Color::RGB(255, 255, 255), vec![700, 10],
        canvas, font, texture_creator
        );
        draw_text(format!("PC: ${:04X}", cpu.registers.pc), Color::RGB(255, 255, 255), vec![700, 10 + font_height],
        canvas, font, texture_creator
        );
        draw_text(format!("A: ${:02X} [{}]", cpu.registers.a, cpu.registers.a), Color::RGB(255, 255, 255), vec![700, 10 + font_height * 2],
        canvas, font, texture_creator
        );
        draw_text(format!("X: ${:02X} [{}]", cpu.registers.x, cpu.registers.x), Color::RGB(255, 255, 255), vec![700, 10 + font_height * 3],
        canvas, font, texture_creator
        );
        draw_text(format!("Y: ${:02X} [{}]", cpu.registers.y, cpu.registers.y), Color::RGB(255, 255, 255), vec![700, 10 + font_height * 4],
        canvas, font, texture_creator
        );
        draw_text(format!("Stack Pointer: $00{:02X}", cpu.registers.sp), Color::RGB(255, 255, 255), vec![700, 10 + font_height * 5],
        canvas, font, texture_creator
        );

        draw_text(format!("SPACE: Step Instruction | R: RESET"), Color::RGB(255, 255, 255), vec![10, 850],
        canvas, font, texture_creator
        );
        
    /*
        let surface = font.render(&debug_text)
        .blended(Color::RGB(255, 255, 255))
        .unwrap();

        let texture = texture_creator
        .create_texture_from_surface(&surface)
        .unwrap();

        let target_rect = Rect::new(
            10,
            10,
            texture.query().width,
            texture.query().height,
        );
        
        canvas.copy(&texture, None, Some(target_rect)).unwrap();
    */
}

fn render_pattern_table<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<sdl2::video::WindowContext>,
    ppu: &PPU,
    rom: &Rom,
    table_idx: u8,
    offset_x: i32,
    offset_y: i32,
) {
    let pattern_table_data = ppu.get_pattern_table(rom, table_idx, 0);

    if let Ok(mut texture) = texture_creator.create_texture_streaming(
        sdl2::pixels::PixelFormatEnum::ARGB8888,
        128, 
        128,
    ) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..128 {
                for x in 0..128 {
                    let color = pattern_table_data[(y * 128 + x) as usize];

                    let offset = y * pitch + x * 4;

                    buffer[offset] = color.b;
                    buffer[offset + 1] = color.g;
                    buffer[offset + 2] = color.r;
                    buffer[offset + 3] = 0xFF;
                }
            }
        }).unwrap();

        let target_rect = Rect::new(
            offset_x, offset_y,
            256, 256,
        );

        canvas.copy(&texture, None, Some(target_rect)).unwrap();
    }
}

fn render_nametable (
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<sdl2::video::WindowContext>,
    ppu: &PPU,
    rom: &Rom,
    ) {
        let bank = (ppu.control >> 4) & 1;
        
        if let Ok(mut texture) = texture_creator.create_texture_streaming(
            PixelFormatEnum::ARGB8888,
            256,
            240,
    ) {
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..30 {
                for x in 0..32 {
                    let tile_idx = ppu.ppu_read(0x2000 + (y * 32) + x, rom) as u16;

                    let bank_offset = bank as u16 * 0x1000;
                    let tile_start = bank_offset + (tile_idx * 16);

                    for row in 0..8 {
                        let plane_0 = ppu.ppu_read(tile_start + row, rom);
                        let plane_1 = ppu.ppu_read(tile_start + row + 8, rom);

                        for col in 0..8 {
                            let bit_0 = (plane_0 >> (7 - col)) & 1;
                            let bit_1 = (plane_1 >> (7 - col)) & 1;
                            let color_idx = (bit_1 << 1) | bit_0;
                            
                            /*
                            let (r, g, b) = match color_idx {
                                0 => (0, 0, 0),
                                1 => (100, 100, 100),
                                2 => (170, 170, 170),
                                3 => (255, 255, 255),
                                _ => (0, 0, 0),
                            };
                            */

                            let color = get_color_from_palette(ppu.ppu_read(0x3F00 + color_idx as  u16, rom));

                            let screen_x = (x * 8 + (col as u16)) as usize;
                            let screen_y = (y * 8 + (row as u16)) as usize;

                            let offset = screen_y * pitch + screen_x * 4;
                            buffer[offset] = color.b;
                            buffer[offset + 1] = color.g;
                            buffer[offset + 2] = color.r;
                            buffer[offset + 3] = 255;
                        }
                    }
                }
            }
        }).unwrap();

        canvas.copy(&texture, None, None).unwrap();
    }
}
