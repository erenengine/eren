use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{Device, FramebufferCreationError, RenderPassCreationError},
    subpass::get_graphic_color_subpass_desc,
    swapchain::Swapchain,
};
use thiserror::Error;

use crate::test_vertex_input::subpass::{TestSubpass, TestSubpassInitializationError};

const CLEAR_VALUES: [vk::ClearValue; 1] = [vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.1921, 0.302, 0.4745, 1.0],
    },
}];

pub struct TestRenderPass {
    device: Arc<Device>,
    render_area: vk::Rect2D,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    subpass: TestSubpass,
}

#[derive(Debug, Error)]
pub enum TestRenderPassInitializationError {
    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] RenderPassCreationError),

    #[error("Failed to create framebuffers: {0}")]
    CreateFramebuffers(#[from] FramebufferCreationError),

    #[error("Failed to create subpass: {0}")]
    CreateSubpass(#[from] TestSubpassInitializationError),
}

impl TestRenderPass {
    pub fn new(
        device: Arc<Device>,
        swapchain: &Swapchain,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestRenderPassInitializationError> {
        let color_attachment = device.get_swapchain_color_attachment_desc();
        let color_attachment_ref = device.get_color_attachment_ref(0);

        let color_refs = [color_attachment_ref];

        // subpass 0
        let subpass_desc = get_graphic_color_subpass_desc(&color_refs);

        let render_pass = device.create_render_pass(
            &[color_attachment],
            &[subpass_desc],
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
                    .dst_access_mask(vk::AccessFlags::MEMORY_READ)
                    .dependency_flags(vk::DependencyFlags::BY_REGION),
            ],
        )?;

        let swapchain_framebuffers = swapchain.create_framebuffers(render_pass)?;
        let subpass = TestSubpass::new(device.clone(), command_pool, render_area, render_pass, 0)?;

        Ok(Self {
            device,
            render_area,
            render_pass,
            swapchain_framebuffers,
            subpass,
        })
    }

    pub fn record_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        swapchain_image_idx: usize,
        frame_idx: usize,
        window_width: u32,
        window_height: u32,
        pre_transform: vk::SurfaceTransformFlagsKHR,
    ) {
        self.device.begin_render_pass(
            command_buffer,
            self.render_pass,
            self.swapchain_framebuffers[swapchain_image_idx],
            self.render_area,
            &CLEAR_VALUES,
        );

        self.subpass.record_commands(
            command_buffer,
            frame_idx,
            window_width,
            window_height,
            pre_transform,
        );
        //self.device.next_subpass(command_buffer); 다음 subpass로 넘어가려면 필요

        self.device.end_render_pass(command_buffer);
    }
}

impl Drop for TestRenderPass {
    fn drop(&mut self) {
        self.device.wait_idle();

        for &framebuffer in self.swapchain_framebuffers.iter() {
            self.device.destroy_framebuffer(framebuffer);
        }

        self.device.destroy_render_pass(self.render_pass);
    }
}
