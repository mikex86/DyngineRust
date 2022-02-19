use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use wgpu::{ColorTargetState, MultisampleState, RenderBundle, RenderBundleDescriptor, RenderBundleEncoderDescriptor, SurfaceConfiguration};
use wgpu::{Color, CommandEncoder, Device};

pub struct EngineCoreState {
    render_pipeline: wgpu::RenderPipeline,
    pub triangle_render_bundle: RenderBundle,
}

pub struct EngineInstance {
    device: Rc<Device>,
    surface_config: Rc<RefCell<SurfaceConfiguration>>,
    color_target_state: ColorTargetState,
    pub multisample_state: MultisampleState,
    engine_core_state: Option<EngineCoreState>,
}

#[derive(Debug, PartialEq)]
pub struct ViewportRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ViewportRegion {
    pub const ZERO: ViewportRegion = ViewportRegion {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };
}

impl EngineInstance {
    pub fn new(device: Rc<Device>, surface_config: Rc<RefCell<SurfaceConfiguration>>) -> EngineInstance {
        let surface_format = surface_config.borrow().format;
        EngineInstance {
            device,
            surface_config,
            color_target_state: ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: Default::default(),
            },
            multisample_state: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            engine_core_state: None,
        }
    }

    #[profiling::function]
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
                targets: &[self.color_target_state.clone()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: self.multisample_state,
            multiview: None,
        });

        let triangle_render_bundle;
        {
            let mut triangle_render_bundle_encoder = self.device.create_render_bundle_encoder(&RenderBundleEncoderDescriptor {
                label: Some("TriangleRenderBundleEncoder"),
                color_formats: &[self.surface_config.borrow_mut().format],
                depth_stencil: None,
                sample_count: self.multisample_state.count,
                multiview: None,
            });
            triangle_render_bundle_encoder.draw(0..3, 0..1);
            triangle_render_bundle = triangle_render_bundle_encoder.finish(&RenderBundleDescriptor {
                label: Some("TriangleRenderBundle"),
            });
        }
        self.engine_core_state = Some(EngineCoreState { render_pipeline, triangle_render_bundle });
    }

    #[profiling::function]
    pub fn render(&self, command_encoder: &mut CommandEncoder, surface_texture_view: &wgpu::TextureView, mutisampled_framebuffer: Option<&wgpu::TextureView>, viewport_region: &ViewportRegion) {
        if viewport_region == &ViewportRegion::ZERO || self.engine_core_state.is_none() {
            return;
        }

        let engine_core_state = self.engine_core_state.as_ref().unwrap();

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MainEngineRenderPass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: if self.multisample_state.count == 1 { &surface_texture_view } else { mutisampled_framebuffer.unwrap() },
                resolve_target: if self.multisample_state.count == 1 { None } else { Some(&surface_texture_view) },
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::TRANSPARENT),
                    // Storing pre-resolve MSAA data is unnecessary if it isn't used later.
                    // On tile-based GPU, avoid store can reduce your app's memory footprint.
                    store: if self.multisample_state.count == 1 { true } else { false },
                },
            }],
            depth_stencil_attachment: None,
        });
        render_pass.set_viewport(viewport_region.x, viewport_region.y, viewport_region.width, viewport_region.height, 0.0, 1.0);
        render_pass.set_pipeline(&engine_core_state.render_pipeline);
        render_pass.execute_bundles(std::iter::once(&engine_core_state.triangle_render_bundle));
    }
}