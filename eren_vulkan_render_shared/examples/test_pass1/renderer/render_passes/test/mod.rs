use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::device::{Device, ShaderModuleCreationError};

mod subpass;

use crate::test_pass1::renderer::render_passes::test::subpass::TestSubpass;

pub struct TestRenderPass {
    //device: Arc<Device>,
    //render_pass: vk::RenderPass,
    subpass: TestSubpass,
}

impl TestRenderPass {
    pub fn new(device: Arc<Device>) -> Result<Self, ShaderModuleCreationError> {
        let subpass = TestSubpass::new(device.clone())?;

        Ok(Self { subpass })
    }
}
