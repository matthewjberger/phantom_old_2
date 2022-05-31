mod texture;

use crate::renderer::Renderer;
use phantom_dependencies::{
    anyhow::{Context, Result},
    egui::{epaint::ClippedMesh, CtxRef},
    egui_wgpu_backend::{RenderPass as GuiRenderPass, ScreenDescriptor},
    log, pollster,
    raw_window_handle::HasRawWindowHandle,
    wgpu::{self, Device, Queue, RenderPipeline, Surface, SurfaceConfiguration},
};
use texture::Texture;

pub struct WgpuRenderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    dimensions: [u32; 2],
    depth_texture: Texture,
    gui_renderpass: GuiRenderPass,
    render_pipeline: RenderPipeline,
}

impl Renderer for WgpuRenderer {
    fn resize(&mut self, dimensions: [u32; 2]) {
        if dimensions[0] == 0 || dimensions[1] == 0 {
            return;
        }
        self.dimensions = dimensions;
        self.config.width = dimensions[0];
        self.config.height = dimensions[1];
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::create_depth_texture(
            &self.device,
            dimensions[0],
            dimensions[1],
            "Depth Texture",
        );
    }

    fn render(&mut self, gui_context: &CtxRef, paint_jobs: Vec<ClippedMesh>) -> Result<()> {
        match self.render_frame(gui_context, paint_jobs) {
            Ok(_) => {}
            // Recreate the swapchain if lost
            Err(wgpu::SurfaceError::Lost) => self.resize(self.dimensions),
            // The system is out of memory, we should probably quit
            // Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors should be resolved by the next frame
            Err(e) => log::error!("{:?}", e),
        }
        Ok(())
    }
}

impl WgpuRenderer {
    pub fn backends() -> wgpu::Backends {
        wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all)
    }

    pub fn new(window_handle: &impl HasRawWindowHandle, dimensions: &[u32; 2]) -> Result<Self> {
        pollster::block_on(WgpuRenderer::new_async(window_handle, dimensions))
    }

    async fn new_async(
        window_handle: &impl HasRawWindowHandle,
        dimensions: &[u32; 2],
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(Self::backends());

        let surface = unsafe { instance.create_surface(window_handle) };

        let adapter = Self::create_adapter(&instance, &surface).await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let swapchain_format = surface
            .get_preferred_format(&adapter)
            .context("Failed to get preferred surface format!")?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: dimensions[0],
            height: dimensions[1],
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let depth_texture =
            Texture::create_depth_texture(&device, dimensions[0], dimensions[1], "Depth Texture");

        let gui_renderpass = GuiRenderPass::new(&device, config.format, 1);

        // Triangle Stuff

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../../assets/shaders/shader.wgsl").into(),
            ),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            dimensions: *dimensions,
            depth_texture,
            gui_renderpass,
            render_pipeline,
        })
    }

    fn required_limits(adapter: &wgpu::Adapter) -> wgpu::Limits {
        wgpu::Limits::default()
            // Use the texture resolution limits from the adapter
            // to support images the size of the surface
            .using_resolution(adapter.limits())
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Result<wgpu::Adapter> {
        wgpu::util::initialize_adapter_from_env_or_default(
            instance,
            Self::backends(),
            Some(surface),
        )
        .await
        .context("No suitable GPU adapters found on the system!")
    }

    async fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
        log::info!("WGPU Adapter Features: {:#?}", adapter.features());

        adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: (Self::optional_features() & adapter.features())
                        | Self::required_features(),
                    limits: Self::required_limits(adapter),
                    label: Some("Render Device"),
                },
                None,
            )
            .await
            .context("Failed to request a device!")
    }

    fn render_frame(
        &mut self,
        gui_context: &CtxRef,
        paint_jobs: Vec<ClippedMesh>,
    ) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let screen_descriptor = ScreenDescriptor {
            physical_width: self.config.width,
            physical_height: self.config.height,
            scale_factor: 1.0, // TODO: Store the scale factor in the renderer and update it when winit reports that the scale factor has changed
        };

        self.gui_renderpass
            .update_texture(&self.device, &self.queue, &gui_context.texture());
        self.gui_renderpass
            .update_user_textures(&self.device, &self.queue);
        self.gui_renderpass.update_buffers(
            &self.device,
            &self.queue,
            &paint_jobs,
            &screen_descriptor,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.push_debug_group("Main Passes");

        encoder.insert_debug_marker("Render Entities");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1);
        }

        encoder.insert_debug_marker("Render Gui");
        self.gui_renderpass
            .execute(&mut encoder, &view, &paint_jobs, &screen_descriptor, None)
            .expect("Failed to execute the gui renderpass!");

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}
