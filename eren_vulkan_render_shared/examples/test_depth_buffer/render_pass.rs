use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    attachment::Attachment,
    command::CommandPool,
    device::{AttachmentCreationError, Device, FramebufferCreationError, RenderPassCreationError},
    physical_device::PhysicalDevice,
    swapchain::Swapchain,
};
use thiserror::Error;

use super::subpass::{TestSubpass, TestSubpassInitializationError};

const CLEAR_VALUES: [vk::ClearValue; 2] = [
    vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.1921, 0.302, 0.4745, 1.0],
        },
    },
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    },
];

pub struct TestRenderPass {
    device: Arc<Device>,
    render_area: vk::Rect2D,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    subpass: TestSubpass,
    depth_attachment: Attachment,
}

#[derive(Debug, Error)]
pub enum TestRenderPassInitializationError {
    #[error("Failed to create depth attachment: {0}")]
    CreateDepthAttachment(#[from] AttachmentCreationError),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] RenderPassCreationError),

    #[error("Failed to create framebuffers: {0}")]
    CreateFramebuffers(#[from] FramebufferCreationError),

    #[error("Failed to create subpass: {0}")]
    CreateSubpass(#[from] TestSubpassInitializationError),
}

impl TestRenderPass {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        swapchain: &Swapchain,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestRenderPassInitializationError> {
        let color_attachment = device.get_swapchain_color_attachment_desc();
        let color_attachment_ref = device.get_color_attachment_ref(0);

        let color_refs = [color_attachment_ref];

        let depth_attachment = device.create_depth_attachment(
            render_area.extent,
            physical_device.depth_format,
            vk::SampleCountFlags::TYPE_1,
            false,
        )?;

        let depth_attachment_ref = device.get_depth_attachment_ref(1);

        // subpass 0
        let subpass_desc = vk::SubpassDescription2::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs)
            .depth_stencil_attachment(&depth_attachment_ref);

        let render_pass = device.create_render_pass(
            &[color_attachment, depth_attachment.desc],
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

        let swapchain_framebuffers = swapchain
            .create_framebuffers_with_depth_image_view(render_pass, depth_attachment.view)?;
        let subpass = TestSubpass::new(device.clone(), command_pool, render_area, render_pass, 0)?;

        Ok(Self {
            device,
            render_area,
            render_pass,
            swapchain_framebuffers,
            subpass,
            depth_attachment,
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

        self.device.destroy_image_view(self.depth_attachment.view);
        self.device
            .destroy_image_with_memory(self.depth_attachment.image, self.depth_attachment.memory);
    }
}
