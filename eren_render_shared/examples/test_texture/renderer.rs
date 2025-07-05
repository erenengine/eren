use eren_render_shared::{adapter::Adapter, device::Device, surface::Surface};
use image::ImageError;

use super::render_pass::TestRenderPass;

pub struct TestRenderer {
    render_pass: TestRenderPass,
}

impl TestRenderer {
    pub fn new(
        adapter: &Adapter,
        device: &Device,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, ImageError> {
        Ok(Self {
            render_pass: TestRenderPass::new(
                device,
                adapter.depth_format,
                window_width,
                window_height,
            )?,
        })
    }

    pub fn resize(
        &mut self,
        device: &Device,
        depth_format: wgpu::TextureFormat,
        window_width: u32,
        window_height: u32,
    ) {
        self.render_pass
            .resize_depth_texture(device, depth_format, window_width, window_height);
    }

    pub fn render(
        &mut self,
        surface: &Surface,
        device: &Device,
        window_width: u32,
        window_height: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Test Render Encoder"),
        });

        self.render_pass
            .record_commands(device, &view, &mut encoder, window_width, window_height);

        device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
