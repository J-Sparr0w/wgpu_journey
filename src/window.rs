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

use crate::primitives::GRID_SIZE;
use crate::render_context::RenderContext;

const FPS_CAP: f32 = 20.;

impl<'a> App<'a> {
    pub fn init(window_attributes: Option<WindowAttributes>) -> App<'a> {
        let window_attributes = match window_attributes {
            Some(w) => w,
            None => WindowAttributes::default(),
        };
        let mut app = App {
            window_attributes,
            window: None,
            render_ctx: None,
            frame_time: Instant::now(),
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
        const WORKGROUP_SIZE: u8 = 8;
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Game of life simulation"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                    @group(0) @binding(0) var<uniform> grid: vec2f;
                    @group(0) @binding(1) var<storage> cell_state_in: array<u32>;
                    @group(0) @binding(2) var<storage, read_write> cell_state_out: array<u32>;

                    fn cellIndex(cell: vec2u) -> u32 {
                        return (cell.y % u32(grid.y)) * u32(grid.x) + (cell.x % u32(grid.x));
                    }
                    fn cellActive(x: u32, y: u32) -> u32 {
                        return cell_state_in[cellIndex(vec2(x, y))];
                    }
                    @compute @workgroup_size(8,8)
                    fn compute_main(@builtin(global_invocation_id) cell: vec3u) {
                    // getting count of active neighbors 
                        let active_neighbors = cellActive(cell.x+1, cell.y+1) +
                            cellActive(cell.x+1, cell.y) +
                            cellActive(cell.x+1, cell.y-1) +
                            cellActive(cell.x, cell.y-1) +
                            cellActive(cell.x-1, cell.y-1) +
                            cellActive(cell.x-1, cell.y) +
                            cellActive(cell.x-1, cell.y+1) +
                            cellActive(cell.x, cell.y+1);

                        let i = cellIndex(cell.xy);
                        switch active_neighbors{
                            case 2u:{
                                cell_state_out[i] = cell_state_in[i];
                            }
                            case 3u:{
                                cell_state_out[i] = 1u;
                            }
                            default:{
                                cell_state_out[i] = 0u;
                            }
                        }
                    }
            "#
                .into(),
            ),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX | ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout], // even though there are two bind groups, only one is used at a time so only one layout is necessary
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

        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &compute_shader,
            entry_point: "compute_main",
            compilation_options: Default::default(),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(crate::primitives::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(crate::primitives::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(crate::primitives::UNIFORM_ARRAY),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let cell_state_array = &mut [0u32; (GRID_SIZE * GRID_SIZE) as usize];
        let mut rng = rand::thread_rng();
        for val in cell_state_array.iter_mut() {
            let rng_val: u8 = rng.gen_range(0..100);
            *val = if rng_val > 60 { 1 } else { 0 };
        }

        let cell_storage_buffers = [
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Cell storage buffer A"),
                contents: bytemuck::cast_slice(cell_state_array),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Cell storage buffer B"),
                contents: bytemuck::cast_slice(cell_state_array),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            }),
        ];

        let bind_groups = [
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Uniform and Storage Bind Group A"),
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: cell_storage_buffers[0].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: cell_storage_buffers[1].as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("Uniform and Storage Bind Group B"),
                layout: &render_pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: cell_storage_buffers[1].as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: cell_storage_buffers[0].as_entire_binding(),
                    },
                ],
            }),
        ];

        let render_ctx = RenderContext::new(
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            compute_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            cell_storage_buffers,
            bind_groups,
        );
        self.render_ctx = Some(render_ctx);
    }
}

pub struct App<'a> {
    window: Option<Arc<Window>>,
    window_attributes: WindowAttributes,
    render_ctx: Option<RenderContext<'a>>,
    frame_time: Instant,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        eprintln!("Resumed");
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
