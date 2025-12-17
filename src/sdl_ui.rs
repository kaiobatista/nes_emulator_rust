use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;

use crate::cpu::CPU;
use crate::ppu::{PPU, get_color_from_palette};
use crate::ines_file::Rom;


pub fn start_ui(mut cpu: CPU) {

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("NES Emulator Debugger", 1200, 900)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let ttf_context = sdl2::ttf::init().unwrap();

    let texture_creator = canvas.texture_creator();

    let font_path = "/usr/share/fonts/TTF/FiraCode-Medium.ttf";
    let mut font = ttf_context.load_font(font_path, 20).unwrap();
    
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running;
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    cpu.step();
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

                _ => {}
            }
        }
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let mut cycles_this_frame = 0;
        while cycles_this_frame < 29780 {
            // 1. Executa 1 instrução da CPU
            let cycles = cpu.step() as usize; 
            cycles_this_frame += cycles;

            // 2. A PPU roda 3 vezes para cada 1 ciclo de CPU
            for _ in 0..(cycles * 3) {
                 cpu.bus.ppu.step();
            }
        }

        cpu.nmi();
        
        //render_debug_info(&mut canvas, &mut font, &texture_creator, &cpu);

        render_pattern_table(&mut canvas, &texture_creator, &cpu.bus.ppu, &cpu.bus.rom, 0, 850, 10);
        render_pattern_table(&mut canvas, &texture_creator, &cpu.bus.ppu, &cpu.bus.rom, 1, 850, 300);

        canvas.present();
    }
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

