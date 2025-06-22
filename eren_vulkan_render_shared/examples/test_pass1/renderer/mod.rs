use std::sync::Arc;

use eren_vulkan_render_shared::{device::{Device, ShaderModuleCreationError}, swapchain::Swapchain};

pub mod render_passes;

pub struct TestRenderer {
    //render_pass: render_passes::test::TestRenderPass,
}

impl TestRenderer {
    pub fn new(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
    ) -> Result<Self, ShaderModuleCreationError> {
        //let render_pass = render_passes::test::TestRenderPass::new(device.clone())?;

        //Ok(Self { render_pass })
        Ok(Self {})
    }

    pub fn render(&self) {}
}
