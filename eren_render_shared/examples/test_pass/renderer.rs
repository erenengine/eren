use eren_render_shared::{device::Device, surface::Surface};

use super::render_pass::TestRenderPass;

pub struct TestRenderer {
    render_pass: TestRenderPass,
}

impl TestRenderer {
    pub fn new(device: &Device) -> Self {
        Self {
            render_pass: TestRenderPass::new(device),
        }
    }

    pub fn render(&self, surface: &Surface, device: &Device) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Test Render Encoder"),
        });

        self.render_pass.record_commands(&view, &mut encoder);

        device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
