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
    render_pass: vk::RenderPass,
    //swapchain_framebuffers: Vec<vk::Framebuffer>,
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

        // subpass 0
        let subpass = get_graphic_color_subpass_desc(&color_refs);

        let render_pass = device.create_render_pass(
            &[color_attachment],
            &[subpass],
            &[
                // external -> subpass 0
                vk::SubpassDependency2::default()
                    .src_subpass(vk::SUBPASS_EXTERNAL)
                    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                    .dst_subpass(0)
                    .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dependency_flags(vk::DependencyFlags::BY_REGION),
                // subpass 0 -> external
                vk::SubpassDependency2::default()
                    .src_subpass(0)
                    .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                    .dst_subpass(vk::SUBPASS_EXTERNAL)
                    .dst_stage_mask(vk::PipelineStageFlags::BOTTOM_OF_PIPE) // 가장 마지막 단계
                    .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dependency_flags(vk::DependencyFlags::BY_REGION),
            ],
        )?;

        Ok(Self { render_pass })
    }
}
