use std::io::Read;
use std::process::Command as StdCommand;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use softbuffer::GraphicsContext;
use global_hotkey::{hotkey::{HotKey, Modifiers}, GlobalHotKeyManager};
use conpty::Process;
use rusttype::{Font, Scale, Point};

struct TerminalState {
    content: String,
    process: Process,
    scroll_offset: usize,
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top(true)
        .build(&event_loop)?;

    let screen_size = window.primary_monitor().unwrap().size();
    let window_height = (screen_size.height / 2) as i32;
    window.set_inner_size(winit::dpi::LogicalSize::new(
        screen_size.width,
        screen_size.height / 2,
    ));
    window.set_outer_position(winit::dpi::LogicalPosition::new(0, -window_height));

    let mut process = Process::spawn(StdCommand::new("cmd.exe"))?;
    let input = process.input()?;
    let output = process.output()?;

    let state = Arc::new(Mutex::new(TerminalState {
        content: String::new(),
        process,
        scroll_offset: 0,
    }));

    let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SUPER), global_hotkey::hotkey::Code::Backquote);
    let manager = GlobalHotKeyManager::new()?;
    manager.register(hotkey)?;

    let mut visible = false;
    let mut animation_start = Instant::now();
    let anim_duration = 0.2;

    let font_data = include_bytes!("C:\\Windows\\Fonts\\consola.ttf");
    let font = Font::try_from_bytes(font_data).unwrap();

    let mut graphics_context = unsafe { GraphicsContext::new(window.clone()) }?;

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        let mut buffer = graphics_context.buffer_mut().unwrap();
                        buffer.fill(0x000000);

                        let state = state.lock().unwrap();
                        let scale = Scale::uniform(24.0);
                        let v_metrics = font.v_metrics(scale);
                        let start_y = v_metrics.ascent;

                        let mut y = start_y;
                        for line in state.content.lines().rev().skip(state.scroll_offset).take(50) {
                            let glyphs: Vec<_> = font.layout(line, scale, Point { x: 10.0, y }).collect();
                            for glyph in glyphs {
                                if let Some(bounds) = glyph.pixel_bounding_box() {
                                    glyph.draw(|x, y, c| {
                                        let alpha = (c * 255.0) as u32;
                                        let color = 0x00FFFFFF | (alpha << 24);
                                        let px = bounds.min.x as usize + x as usize;
                                        let py = bounds.min.y as usize + y as usize;
                                        if px < buffer.width() && py < buffer.height() {
                                            buffer[py * buffer.width() + px] = color;
                                        }
                                    });
                                }
                            }
                            y += v_metrics.line_gap + v_metrics.ascent - v_metrics.descent;
                        }

                        buffer.present().unwrap();
                    }
                    _ => (),
                }
            }
            Event::AboutToWait => {
                window.request_redraw();

                // Read terminal output
                let mut buf = [0; 1024];
                if let Ok(n) = output.read(&mut buf) {
                    let new_content = String::from_utf8_lossy(&buf[..n]);
                    state.lock().unwrap().content.push_str(&new_content);
                }

                // Check hotkey
                if manager.poll().contains(&hotkey.id()) {
                    visible = !visible;
                    animation_start = Instant::now();
                }

                // Animate
                if animation_start.elapsed().as_secs_f32() < anim_duration {
                    let t = animation_start.elapsed().as_secs_f32() / anim_duration;
                    let y_pos = if visible {
                        -window_height as f32 + t * window_height as f32
                    } else {
                        -t * window_height as f32
                    };
                    window.set_outer_position(winit::dpi::LogicalPosition::new(0, y_pos as i32));
                }
            }
            _ => (),
        }
    });
    Ok(())
}