use egui::{RawInput, Context};
use winit::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};
use wgpu::{Instance, Device, Queue, RequestAdapterOptions};
use egui_wgpu::Renderer;
use egui_winit::{State, egui, egui::viewport::ViewportId};

use std::process;

async fn run(event_loop: EventLoop<()>, window: winit::window::Window) {
    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })
    .await
    .expect("Failed to find an adapter");

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
    }, None)
    .await
    .expect("Failed to create device and queue");

    let mut state = State::new(
        egui::Context::default(),
        ViewportId::ROOT,
        &window,
        None,
        None,
    );

    // Initialize the renderer with the device
    let mut renderer = Renderer::new(&device, wgpu::TextureFormat::Bgra8UnormSrgb, None, 1);

    // Event loop
    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    std::process::exit(0);
                }
                WindowEvent::RedrawRequested => {
                    // Create an empty RawInput if you don't have actual input handling yet
                    let raw_input = RawInput::default();

                    // Make sure the fonts are initialized by accessing the context
                    state.egui_ctx().run(raw_input, move |ctx| {
                        egui::CentralPanel::default().show(ctx, |ui| {
                            ui.label("Hello from Egui!");
                            if ui.button("Close").clicked() {
                                std::process::exit(0);
                            }
                        });
                    });

                    window.request_redraw(); // Request a redraw to update the window
                }
                _ => {}
            },
            _ => {}
        }

        // Control flow is handled by event_loop.run() itself.
    });
}

fn main() {
    // Create the event loop (unwrap the result)
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    
    // Build the window (unwrap the result)
    let window = WindowBuilder::new()
        .with_title("Egui + WGPU")
        .build(&event_loop)
        .expect("Failed to create window");

    // Run the application with the unwrapped event loop and window
    pollster::block_on(run(event_loop, window));
}
