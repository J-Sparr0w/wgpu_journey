use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::Rng;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use wgpu::{
    Adapter, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    ComputePipelineDescriptor, Device, Queue, RequestDeviceError, ShaderStages, Surface,
    VertexAttribute, VertexBufferLayout, VertexFormat,
};

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    // keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes},
};

use crate::render_context::RenderContext;
use crate::render_state::RenderState;

const FPS_CAP: f32 = 20.;

impl<'a> App<'a> {
    pub fn init(
        window_attributes: Option<WindowAttributes>,
        render_state: Option<RenderState>,
    ) -> App<'a> {
        let window_attributes = match window_attributes {
            Some(w) => w,
            None => WindowAttributes::default(),
        };
        let mut app = App {
            window_attributes,
            window: None,
            render_ctx: None,
            frame_time: Instant::now(),
            render_state,
        };

        app.init_eventloop_and_window();

        app
    }
    fn init_eventloop_and_window(&mut self) -> () {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self).unwrap();
    }
    pub fn init_renderer(&mut self) {
        eprintln!("init_renderer");
        let window = self
            .window
            .as_ref()
            .expect("ERROR: No window found.")
            .clone();
        let size = self
            .window
            .as_ref()
            .expect("ERROR: No window found.")
            .inner_size();
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface: Surface<'_> = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(request_adapter(&instance, &surface)).unwrap();

        let (device, queue) =
            pollster::block_on(request_device(&adapter)).expect("ERROR: setting up device failed.");

        //
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0]; //first format is the preferred format
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[], // even though there are two bind groups, only one is used at a time so only one layout is necessary
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format.into(),
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(self.render_state.as_ref().unwrap().vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Uniform Buffer"),
        //     contents: bytemuck::cast_slice(crate::primitives::UNIFORM_ARRAY),
        //     usage: wgpu::BufferUsages::UNIFORM,
        // });

        let render_ctx = RenderContext::new(
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            Some(vertex_buffer),
            None,
            None,
            None,
            std::mem::take(&mut self.render_state).unwrap(),
        );
        self.render_ctx = Some(render_ctx);
    }
}

pub struct App<'a> {
    window: Option<Arc<Window>>,
    window_attributes: WindowAttributes,
    render_ctx: Option<RenderContext<'a>>,
    frame_time: Instant,
    render_state: Option<RenderState>,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // eprintln!("Resumed");
        let window_attributes = self.window_attributes.clone();
        self.window = Some(Arc::new(
            event_loop.create_window(window_attributes).unwrap(),
        ));
        self.init_renderer();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self
                    .window
                    .as_ref()
                    .expect("ERROR: A window should be present")
                    .id()
                    != window_id
                {
                    return;
                }
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                if let Some(ctx) = self.render_ctx.as_mut() {
                    let frame_time = self.frame_time.elapsed();
                    if frame_time.as_secs_f32() >= 1. / FPS_CAP {
                        // println!("Time: {:?}", self.frame_time.elapsed());
                        ctx.render();
                        self.frame_time = Instant::now();
                    }
                }

                self.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::Resized(new_size) => {
                self.render_ctx.as_mut().unwrap().resize(new_size);
            }
            _ => (),
        }
    }

    fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        println!("event: exiting events");
    }

    fn memory_warning(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let _ = event_loop;
        println!("event: memory_warning events");
    }
}

//utility functions
pub async fn request_adapter<'a>(
    instance: &wgpu::Instance,
    surface: &Surface<'a>,
) -> Option<Adapter> {
    instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
}

pub async fn request_device<'a>(adapter: &Adapter) -> Result<(Device, Queue), RequestDeviceError> {
    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
}
