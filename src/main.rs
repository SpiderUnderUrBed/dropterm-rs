use crossbeam_channel::unbounded;
use win_hotkeys::{HotkeyManager, VKey};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{WindowBuilder},
};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy)]
enum CustomEvent {
    ToggleVisibility,
}

fn main() {
    // Create the event loop using the new builder method
    let event_loop: EventLoop<CustomEvent> = EventLoopBuilder::with_user_event().build();
    let event_proxy = event_loop.create_proxy(); // Proxy for sending events

    let mut hkm = HotkeyManager::new();

    // Set up the hotkey manager with a crossbeam channel
    let (tx, _rx) = unbounded();  // _rx is unused, so prefix with _
    hkm.register_channel(tx);

    let backquote = VKey::from_vk_code(0xC0);

    // Use Arc and clone the necessary values inside the closure
    let last_time = Arc::new(Mutex::new(std::time::Instant::now()));

    // Register hotkeys
    hkm.register_hotkey(backquote, &[VKey::Control], {
        let last_time = Arc::clone(&last_time);
        let event_proxy = event_proxy.clone();
        move || {
            let now = std::time::Instant::now();
            let mut last_time_guard = last_time.lock().unwrap();
            
            if now.duration_since(*last_time_guard) > std::time::Duration::from_millis(300) {
                *last_time_guard = now;
                println!("Ctrl + ` hotkey pressed");
                let _ = event_proxy.send_event(CustomEvent::ToggleVisibility); // Send event through proxy
            }
        }
    })
    .expect("Failed to register Ctrl+` hotkey");

    hkm.register_hotkey(backquote, &[VKey::LWin], {
        let last_time = Arc::clone(&last_time);
        let event_proxy = event_proxy.clone();
        move || {
            let now = std::time::Instant::now();
            let mut last_time_guard = last_time.lock().unwrap();

            if now.duration_since(*last_time_guard) > std::time::Duration::from_millis(300) {
                *last_time_guard = now;
                println!("Meta + ` hotkey pressed");
                let _ = event_proxy.send_event(CustomEvent::ToggleVisibility); // Send event through proxy
            }
        }
    })
    .expect("Failed to register Meta+` hotkey");

    // Create the window
    let window = WindowBuilder::new()
        .with_title("Quake Terminal")
        .with_decorations(false)
        .with_transparent(true)
        .with_inner_size(winit::dpi::LogicalSize::new(800, 400))
        .build(&event_loop)
        .unwrap();

    window.set_visible(true);
    let mut visible = true;

    // Spawn a thread to run the hotkey manager event loop
    thread::spawn(move || {
        hkm.event_loop();
    });

    // Event loop to handle window events and custom events
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // Handle the custom event to toggle visibility
            Event::UserEvent(CustomEvent::ToggleVisibility) => {
                visible = !visible;
                println!("Toggling visibility: {}", visible);
                window.set_visible(visible);
            }
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => (),
        }
    });
}
