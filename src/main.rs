    use crossbeam_channel::unbounded;
    use win_hotkeys::{HotkeyManager, VKey};
    use winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
        window::{WindowBuilder},
    };
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::ffi::c_void;
    use winit::platform::windows::WindowExtWindows;

    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Gdi::{
        BeginPaint, EndPaint, FillRect, PAINTSTRUCT, CreateSolidBrush, DeleteObject,
    };


    #[derive(Debug, Clone, Copy)]
    enum CustomEvent {
        ToggleVisibility,
    }

    fn main() {
        
        let event_loop: EventLoop<CustomEvent> = EventLoopBuilder::with_user_event().build();
        let event_proxy = event_loop.create_proxy(); 

        let mut hkm = HotkeyManager::new();

        
        let (tx, _rx) = unbounded();  
        hkm.register_channel(tx);

        let backquote = VKey::from_vk_code(0xC0);

        
        let last_time = Arc::new(Mutex::new(std::time::Instant::now()));

        
        hkm.register_hotkey(backquote, &[VKey::Control], {
            let last_time = Arc::clone(&last_time);
            let event_proxy = event_proxy.clone();
            move || {
                let now = std::time::Instant::now();
                let mut last_time_guard = last_time.lock().unwrap();
                
                if now.duration_since(*last_time_guard) > std::time::Duration::from_millis(300) {
                    *last_time_guard = now;
                    println!("Ctrl +  hotkey pressed");
                    let _ = event_proxy.send_event(CustomEvent::ToggleVisibility); 
                }
            }
        })
        .expect("Failed to register Ctrl+ hotkey");

        hkm.register_hotkey(backquote, &[VKey::LWin], {
            let last_time = Arc::clone(&last_time);
            let event_proxy = event_proxy.clone();
            move || {
                let now = std::time::Instant::now();
                let mut last_time_guard = last_time.lock().unwrap();

                if now.duration_since(*last_time_guard) > std::time::Duration::from_millis(300) {
                    *last_time_guard = now;
                    println!("Meta +  hotkey pressed");
                    let _ = event_proxy.send_event(CustomEvent::ToggleVisibility); 
                }
            }
        })
        .expect("Failed to register Meta+ hotkey");

        
        let window = WindowBuilder::new()
            .with_title("Quake Terminal")
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size(winit::dpi::LogicalSize::new(800, 400))
            .build(&event_loop)
            .unwrap();

        window.set_visible(true);
        let mut visible = true;

        
        thread::spawn(move || {
            hkm.event_loop();
        });

        
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                
                Event::UserEvent(CustomEvent::ToggleVisibility) => {
                    visible = !visible;
                    println!("Toggling visibility: {}", visible);
                    window.set_visible(visible);
                    if visible {
                    
                    }
                }
                Event::WindowEvent { event, .. } => {
                    if let WindowEvent::CloseRequested = event {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::RedrawRequested(_) => {
                    // Cast the window handle to HWND (which is a type alias, not a constructor)
                    let hwnd = window.hwnd() as HWND;
                    // Initialize PAINTSTRUCT using zeroed memory
                    let mut ps: PAINTSTRUCT = unsafe { std::mem::zeroed() };
                    unsafe {
                        let hdc = BeginPaint(hwnd, &mut ps);
                        // Use a u32 literal for COLORREF instead of calling a constructor
                        let hbr = CreateSolidBrush(0x2ba5u32);
                        FillRect(hdc, &ps.rcPaint, hbr);
                        let _ = DeleteObject(hbr as _);
                        EndPaint(hwnd, &ps);
                    }
                },
                
                _ => (),
            }
        });
    }