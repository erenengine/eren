use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    device::{Device, GraphicsPipelineCreationError, RenderPassCreationError},
    subpass::get_graphic_color_subpass_desc,
};
use thiserror::Error;

mod subpass;

use crate::test_pass1::renderer::render_passes::test::subpass::TestSubpass;

pub struct TestRenderPass {
    //device: Arc<Device>,
    //render_pass: vk::RenderPass,
    //subpass: TestSubpass,
}

#[derive(Debug, Error)]
pub enum TestRenderPassCreationError {
    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(#[from] RenderPassCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    GraphicsPipelineCreationFailed(#[from] GraphicsPipelineCreationError),
}

impl TestRenderPass {
    pub fn new(device: Arc<Device>) -> Result<Self, TestRenderPassCreationError> {
        let color_attachment = device.get_swapchain_color_attachment_desc();
        let color_attachment_ref = device.get_color_attachment_ref(0);

        let color_refs = [color_attachment_ref];
        let subpass = get_graphic_color_subpass_desc(&color_refs);

        let render_pass = device.create_render_pass(&[color_attachment], &[subpass], &[])?;

        Ok(Self {})
    }
}
