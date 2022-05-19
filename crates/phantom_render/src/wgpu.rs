use crate::{renderer::Renderer, texture::Texture};
use phantom_dependencies::{
    anyhow::{Context, Result},
    log, pollster,
    raw_window_handle::HasRawWindowHandle,
    wgpu::{self, Device, Queue, Surface, SurfaceConfiguration},
};

pub struct WgpuRenderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    dimensions: [u32; 2],
    depth_texture: Texture,
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

    fn render(&mut self, dimensions: &[u32; 2]) -> Result<()> {
        match self.render_frame(dimensions) {
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

        Ok(Self {
            surface,
            device,
            queue,
            config,
            dimensions: *dimensions,
            depth_texture,
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

    fn render_frame(&mut self, _dimensions: &[u32; 2]) -> Result<(), wgpu::SurfaceError> {
        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        encoder.push_debug_group("Main Passes");

        encoder.insert_debug_marker("Render Entities");
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}
