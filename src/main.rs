use crossbeam_channel::unbounded;
use win_hotkeys::{HotkeyManager, VKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::thread;

#[derive(Debug, Clone, Copy)]
enum CustomEvent {
    ToggleVisibility,
}

fn main() {
    let event_loop: EventLoop<CustomEvent> = EventLoop::with_user_event();
    let _event_proxy = event_loop.create_proxy();

    let mut hkm = HotkeyManager::new();

    let (tx, rx) = unbounded();
    hkm.register_channel(tx);

    let backquote = VKey::from_vk_code(0xC0);

    println!("Registering Ctrl + ` hotkey...");
    let result = hkm.register_hotkey(backquote, &[VKey::Control], || {
        println!("Ctrl + ` hotkey pressed");
        CustomEvent::ToggleVisibility
    });
    println!("Registration result: {:?}", result);

    println!("Registering Meta + ` hotkey...");
    let result = hkm.register_hotkey(backquote, &[VKey::LWin], || {
        println!("Meta + ` hotkey pressed");
        CustomEvent::ToggleVisibility
    });
    println!("Registration result: {:?}", result);

    let window = WindowBuilder::new()
        .with_title("Quake Terminal")
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 400))
        .build(&event_loop)
        .unwrap();

    window.set_visible(true);
    let mut visible = true;

    // Ensure the hotkey manager runs in a separate thread
    thread::spawn(move || {
        println!("Starting hotkey manager event loop...");
        hkm.event_loop();
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                while let Ok(hotkey_event) = rx.try_recv() {
                    println!("Hotkey event received");
                    match hotkey_event {
                        CustomEvent::ToggleVisibility => {
                            visible = !visible;
                            println!("Toggling visibility: {}", visible);
                            window.set_visible(visible);
                        }
                    }
                }
            }
            _ => (),
        }
    });
}
