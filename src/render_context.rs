use std::{iter, primitive};

use wgpu::{
    util::DeviceExt, Adapter, BindGroup, Buffer, BufferSlice, Color, CommandEncoderDescriptor,
    ComputePassDescriptor, Device, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestDeviceError, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::{
    primitives::{Point2, GRID_SIZE, VERTICES},
    render_state::RenderState,
};

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    uniform_buffer: Option<Buffer>,
    storage_buffers: Option<[Buffer; 2]>,
    bind_groups: Option<[BindGroup; 2]>,
}

pub struct RenderContext<'a> {
    render_state: RenderState,
    renderer: Renderer<'a>,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        surface: Surface<'a>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
        size: winit::dpi::PhysicalSize<u32>,
        render_pipeline: wgpu::RenderPipeline,
        vertex_buffer: Option<Buffer>,
        uniform_buffer: Option<Buffer>,
        storage_buffers: Option<[Buffer; 2]>,
        bind_groups: Option<[BindGroup; 2]>,
        render_state: RenderState,
    ) -> RenderContext<'_> {
        let index_buffer = Some(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(render_state.indices.as_slice()),
                usage: wgpu::BufferUsages::INDEX,
            }),
        );

        RenderContext {
            renderer: Renderer {
                surface,
                device,
                queue,
                config,
                size,
                render_pipeline,
                vertex_buffer,
                index_buffer,
                uniform_buffer,
                storage_buffers,
                bind_groups,
            },
            render_state,
        }
    }

    pub fn init_index_buffer(&mut self) {}

    pub fn index_buffer_slice(&self) -> BufferSlice<'_> {
        self.renderer.index_buffer.as_ref().unwrap().slice(..)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.renderer.size = new_size;
            self.renderer.config.width = new_size.width;
            self.renderer.config.height = new_size.height;
            self.renderer
                .surface
                .configure(&self.renderer.device, &self.renderer.config);
        }
    }

    pub fn render(&mut self) {
        let curr_surface_texture = self.renderer.surface.get_current_texture().unwrap();
        let surface_texture_view = curr_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .renderer
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &surface_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 10. / 255.,
                            g: 12. / 255.,
                            b: 28. / 255.,
                            a: 1.,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.renderer.render_pipeline);
            if let Some(buf) = self.renderer.vertex_buffer.as_ref() {
                pass.set_vertex_buffer(0, buf.slice(..));
            }
            pass.set_index_buffer(self.index_buffer_slice(), wgpu::IndexFormat::Uint16);
            // pass.set_bind_group(0, &self.renderer.bind_groups[self.renderer.step as usize], &[]);

            // dbg!(self.render_state.indices.len());
            // dbg!(self.render_state.primitives_count);
            pass.draw_indexed(
                0..(self.render_state.indices.len() as u32),
                0,
                0..(self.render_state.primitives_count) as u32,
            );

            // pass.draw(
            //     0..(self.render_state.vertices.len() / 2) as u32,
            //     0..(self.render_state.primitives_count),
            // );
        }
        self.renderer.queue.submit(iter::once(encoder.finish()));
        curr_surface_texture.present();
    }
}
