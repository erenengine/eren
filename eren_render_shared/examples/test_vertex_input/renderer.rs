use eren_render_shared::{device::Device, surface::Surface};

use crate::test_vertex_input::render_pass::TestRenderPass;

pub struct TestRenderer {
    render_pass: TestRenderPass,
}

impl TestRenderer {
    pub fn new(device: &Device) -> Self {
        Self {
            render_pass: TestRenderPass::new(device),
        }
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
