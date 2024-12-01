use std::{iter, primitive};

use wgpu::{
    Adapter, BindGroup, Buffer, BufferSlice, Color, CommandEncoderDescriptor,
    ComputePassDescriptor, Device, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RequestDeviceError, Surface, SurfaceConfiguration, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

use crate::primitives::GRID_SIZE;

pub struct RenderContext<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    storage_buffers: [Buffer; 2],
    bind_groups: [BindGroup; 2],
    step: u8,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        surface: Surface<'a>,
        device: Device,
        queue: Queue,
        config: SurfaceConfiguration,
        size: winit::dpi::PhysicalSize<u32>,
        render_pipeline: wgpu::RenderPipeline,
        compute_pipeline: wgpu::ComputePipeline,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
        uniform_buffer: Buffer,
        storage_buffers: [Buffer; 2],
        bind_groups: [BindGroup; 2],
    ) -> RenderContext<'_> {
        RenderContext {
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
            storage_buffers,
            bind_groups,
            step: 0,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        self.step = (self.step + 1) % 2;
        // dbg!(self.step);
        let curr_surface_texture = self.surface.get_current_texture().unwrap();
        let surface_texture_view = curr_surface_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        {
            {
                let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });

                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(0, &self.bind_groups[(self.step) as usize], &[]);
                const WORKGROUP_SIZE: u8 = 8;
                let workgroup_count = (GRID_SIZE / WORKGROUP_SIZE as f32).ceil() as u32;
                compute_pass.dispatch_workgroups(workgroup_count, workgroup_count, 1);
            }
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
            pass.set_pipeline(&self.render_pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_bind_group(0, &self.bind_groups[self.step as usize], &[]);
            pass.draw_indexed(0..6, 0, 0..(GRID_SIZE * GRID_SIZE) as u32);
        }
        self.queue.submit(iter::once(encoder.finish()));
        curr_surface_texture.present();
    }
}
