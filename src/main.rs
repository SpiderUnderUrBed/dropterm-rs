use std::process;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu::{Instance, RequestAdapterOptions, StoreOp};
use egui_winit::State;
use egui;
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

async fn run(event_loop: EventLoop<()>, window: Arc<winit::window::Window>) {
    // WGPU Initialization
    let instance = Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let surface = unsafe { instance.create_surface(&window).unwrap() };

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats[0];
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Egui Initialization
    let mut state = State::new(
        egui::Context::default(),
        egui::ViewportId::ROOT,
        &window,
        None,
        None,
    );

    let mut renderer = Renderer::new(
        &device,
        surface_format,
        None,
        1
    );

    let window_clone = Arc::clone(&window);
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window_clone.id() => {
                match event {
                    WindowEvent::CloseRequested => process::exit(0),
                    WindowEvent::Resized(new_size) => {
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config);
                        window_clone.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        // Handle rendering
                        let raw_input = state.take_egui_input(&window_clone);
                        let full_output = state.egui_ctx().run(raw_input, |ctx| {
                            egui::CentralPanel::default().show(ctx, |ui| {
                                ui.label("Hello from Egui!");
                                if ui.button("Close").clicked() {
                                    process::exit(0);
                                }
                            });
                        });

                        let paint_jobs = state.egui_ctx().tessellate(
                            full_output.shapes,
                            window_clone.scale_factor() as f32
                        );

                        let textures_delta = full_output.textures_delta;
                        let frame = surface.get_current_texture().unwrap();
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                        
                        let screen_descriptor = ScreenDescriptor {
                            size_in_pixels: [config.width, config.height],
                            pixels_per_point: window_clone.scale_factor() as f32,
                        };
                        
                        renderer.update_buffers(
                            &device,
                            &queue,
                            &mut encoder,
                            &paint_jobs,
                            &screen_descriptor,
                        );
                        
                        for (id, image_delta) in &textures_delta.set {
                            renderer.update_texture(&device, &queue, *id, image_delta);
                        }

                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("egui_render_pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                    store: StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                        renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        drop(render_pass);

                        queue.submit(Some(encoder.finish()));
                        frame.present();
                    }
                    _ => {
                        let response = state.on_window_event(&window_clone, &event);
                        if response.consumed {
                            return;
                        }
                    }
                }
            }
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Egui + WGPU")
            .build(&event_loop)
            .unwrap()
    );

    pollster::block_on(run(event_loop, window));
}