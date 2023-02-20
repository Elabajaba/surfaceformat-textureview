use std::borrow::Cow;

// use std::borrow::Cow;
use wgpu::{ColorTargetState, ColorWrites, TextureViewDescriptor};
use winit::event_loop::EventLoop;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    // Initialize wgpu
    let event_loop: EventLoop<()> = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window).unwrap() };
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
    let (device, queue) =
        pollster::block_on(adapter.request_device(&Default::default(), None)).unwrap();
    
    // ==============================================================================================
    // CHANGE THIS in case it crashes with an unsupported format.
    // ==============================================================================================
    let swapchain_format = wgpu::TextureFormat::Bgra8Unorm;
    // let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb;

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![swapchain_format.add_srgb_suffix()],
    };
    surface.configure(&device, &config);

    // Prepare scene
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(ColorTargetState {
                format: config.view_formats[0], // <------ To get an sRGB view of a non-sRGB texture we need to use the sRGB format here
                blend: None,
                write_mask: ColorWrites::all(),
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Recreate the swap chain with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
            }
            Event::RedrawRequested(_) => {
                let output_frame = surface.get_current_texture().unwrap();
                // let output_view = output_frame.texture.create_view(&Default::default()); // This won't work to get an sRGB view of a non-sRGB surface
                let output_view = output_frame.texture.create_view(&TextureViewDescriptor {
                    format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),  // <------ This will work
                    ..Default::default()
                });

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &output_view, // <------ Plug the sRGB view in here
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });
                    rpass.set_pipeline(&render_pipeline);
                    rpass.draw(0..3, 0..1);
                }
                queue.submit(Some(encoder.finish()));

                output_frame.present();
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {
                window.request_redraw();
            }
        }
    });
}
