//! The parts of this example enabling MSAA are:
//! *    The render pipeline is created with a sample_count > 1.
//! *    A new texture with a sample_count > 1 is created and set as the color_attachment instead of the swapchain.
//! *    The swapchain is now specified as a resolve_target.
//!
//! The parts of this example enabling LineList are:
//! *   Set the primitive_topology to PrimitiveTopology::LineList.
//! *   Vertices and Indices describe the two points that make up a line.

#[path = "../framework.rs"]
mod framework;

use std::iter;

use bytemuck::{Pod, Zeroable};

use wgpu::vertex_attr_array;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    _pos: [f32; 2],
    _color: [f32; 4],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

struct Example {
    bundle: wgpu::RenderBundle,
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    multisampled_framebuffer: wgpu::TextureView,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    sample_count: u32,
    rebuild_bundle: bool,
    sc_desc: wgpu::SwapChainDescriptor,
}

impl Example {
    fn create_bundle(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        vs_module: &wgpu::ShaderModule,
        fs_module: &wgpu::ShaderModule,
        pipeline_layout: &wgpu::PipelineLayout,
        sample_count: u32,
        vertex_buffer: &wgpu::Buffer,
        vertex_count: u32,
    ) -> wgpu::RenderBundle {
        log::info!("sample_count: {}", sample_count);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::LineList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float2, 1 => Float4],
                }],
            },
            sample_count,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
        let mut encoder =
            device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: None,
                color_formats: &[sc_desc.format],
                depth_stencil_format: None,
                sample_count,
            });
        encoder.set_pipeline(&pipeline);
        encoder.set_vertex_buffer(0, vertex_buffer.slice(..));
        encoder.draw(0..vertex_count, 0..1);
        encoder.finish(&wgpu::RenderBundleDescriptor {
            label: Some("main"),
        })
    }

    fn create_multisampled_framebuffer(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count: sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: None,
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_default_view()
    }
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        log::info!("Press left/right arrow keys to change sample_count.");
        let sample_count = 4;

        let vs_bytes = include_bytes!("shader.vert.spv");
        let fs_bytes = include_bytes!("shader.frag.spv");
        let vs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs_bytes[..])).unwrap());
        let fs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs_bytes[..])).unwrap());

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[],
        });

        let multisampled_framebuffer =
            Example::create_multisampled_framebuffer(device, sc_desc, sample_count);

        let mut vertex_data = vec![];

        let max = 50;
        for i in 0..max {
            let percent = i as f32 / max as f32;
            let (sin, cos) = (percent * 2.0 * std::f32::consts::PI).sin_cos();
            vertex_data.push(Vertex {
                _pos: [0.0, 0.0],
                _color: [1.0, -sin, cos, 1.0],
            });
            vertex_data.push(Vertex {
                _pos: [1.0 * cos, 1.0 * sin],
                _color: [sin, -cos, 1.0, 1.0],
            });
        }

        let vertex_buffer = device.create_buffer_with_data(
            bytemuck::cast_slice(&vertex_data),
            wgpu::BufferUsage::VERTEX,
        );
        let vertex_count = vertex_data.len() as u32;

        let bundle = Example::create_bundle(
            device,
            &sc_desc,
            &vs_module,
            &fs_module,
            &pipeline_layout,
            sample_count,
            &vertex_buffer,
            vertex_count,
        );

        let this = Example {
            bundle,
            vs_module,
            fs_module,
            pipeline_layout,
            multisampled_framebuffer,
            vertex_buffer,
            vertex_count,
            sample_count,
            rebuild_bundle: false,
            sc_desc: sc_desc.clone(),
        };
        (this, None)
    }

    fn update(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if let winit::event::ElementState::Pressed = input.state {
                    match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Left) => {
                            if self.sample_count >= 2 {
                                self.sample_count = self.sample_count >> 1;
                                self.rebuild_bundle = true;
                            }
                        }
                        Some(winit::event::VirtualKeyCode::Right) => {
                            if self.sample_count <= 16 {
                                self.sample_count = self.sample_count << 1;
                                self.rebuild_bundle = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.sc_desc = sc_desc.clone();
        self.multisampled_framebuffer =
            Example::create_multisampled_framebuffer(device, sc_desc, self.sample_count);
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> wgpu::CommandBuffer {
        if self.rebuild_bundle {
            self.bundle = Example::create_bundle(
                device,
                &self.sc_desc,
                &self.vs_module,
                &self.fs_module,
                &self.pipeline_layout,
                self.sample_count,
                &self.vertex_buffer,
                self.vertex_count,
            );
            self.multisampled_framebuffer =
                Example::create_multisampled_framebuffer(device, &self.sc_desc, self.sample_count);
            self.rebuild_bundle = false;
        }

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let rpass_color_attachment = if self.sample_count == 1 {
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }
            } else {
                wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.multisampled_framebuffer,
                    resolve_target: Some(&frame.view),
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color::BLACK,
                }
            };

            encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[rpass_color_attachment],
                    depth_stencil_attachment: None,
                })
                .execute_bundles(iter::once(&self.bundle));
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("msaa-line");
}
