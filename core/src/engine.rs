use std::borrow::Cow;
use std::rc::Rc;
use egui::Rect;
use egui_wgpu_backend::wgpu::TextureFormat;
use wgpu::{Color, CommandEncoder, Device};

pub struct EngineCoreState {
    render_pipeline: wgpu::RenderPipeline,
}

pub struct EngineInstance {
    device: Rc<Device>,
    surface_format: wgpu::TextureFormat,
    engine_core_state: Option<EngineCoreState>
}

impl EngineInstance {

    pub fn new(device: Rc<Device>, surface_format: TextureFormat) -> EngineInstance {
        EngineInstance { device, surface_format, engine_core_state: None }
    }

    pub fn start(&mut self) {
        let shader = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../cres/shaders/shader.frag.wgsl"))),
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                targets: &[self.surface_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        self.engine_core_state = Some(EngineCoreState { render_pipeline });
    }

    pub fn render(&self, command_encoder: &mut CommandEncoder, surface_texture_view: &wgpu::TextureView, viewport_region: Rect) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        if viewport_region == Rect::NOTHING {
            return;
        }
        render_pass.set_viewport(viewport_region.min.x, viewport_region.min.y, viewport_region.max.x, viewport_region.max.y, 0.0, 1.0);

        let engine_core_state = self.engine_core_state.as_ref().unwrap();

        render_pass.set_pipeline(&engine_core_state.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }
}