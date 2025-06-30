use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        CommandBufferBeginError, CommandBufferEndError, CommandBufferResetError, Device,
        ImageViewCreationError, ImageWithMemoryCreationError, ResetFencesError,
        SubmitGraphicsCommandsError, WaitForFencesError,
    },
    frame::{FrameManager, FrameManagerInitializationError},
    physical_device::PhysicalDevice,
    swapchain::{Swapchain, SwapchainAcquireError, SwapchainPresentError},
};
use thiserror::Error;

use crate::test_shadow::{
    mesh::MeshBuffer,
    shadow_pass::{TestShadowPass, TestShadowPassInitializationError},
    ubo::ShadowUBO,
};

use super::debug_quad_pass::{DebugQuadPass, DebugQuadPassInitializationError};

pub struct TestRenderer {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    frame_mgr: FrameManager,
    shadow_pass: TestShadowPass,
    debug_quad_pass: DebugQuadPass,
}

#[derive(Debug, Error)]
pub enum TestRendererInitializationError {
    #[error("Failed to create frame manager: {0}")]
    CreateFrameManager(#[from] FrameManagerInitializationError),

    #[error("Failed to create image with memory: {0}")]
    CreateImageWithMemory(#[from] ImageWithMemoryCreationError),

    #[error("Failed to create image view: {0}")]
    CreateImageView(#[from] ImageViewCreationError),

    #[error("Failed to create shadow pass: {0}")]
    CreateShadowPass(#[from] TestShadowPassInitializationError),

    #[error("Failed to create render pass: {0}")]
    CreateDebugQuadPass(#[from] DebugQuadPassInitializationError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to wait for fences: {0}")]
    WaitForFences(#[from] WaitForFencesError),

    #[error("Failed to reset fences: {0}")]
    ResetFences(#[from] ResetFencesError),

    #[error("Failed to reset command buffer: {0}")]
    ResetCommandBuffer(#[from] CommandBufferResetError),

    #[error("Failed to begin command buffer: {0}")]
    BeginCommandBuffer(#[from] CommandBufferBeginError),

    #[error("Failed to end command buffer: {0}")]
    EndCommandBuffer(#[from] CommandBufferEndError),

    #[error("Failed to acquire next image: {0}")]
    AcquireNextImage(#[from] SwapchainAcquireError),

    #[error("Failed to submit graphics commands: {0}")]
    SubmitGraphicsCommands(#[from] SubmitGraphicsCommandsError),

    #[error("Failed to present: {0}")]
    Present(#[from] SwapchainPresentError),
}

pub fn transition_image_layout(
    device: &Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    aspect_mask: vk::ImageAspectFlags,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    device.pipeline_barrier(
        cmd,
        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
    );
}

impl TestRenderer {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestRendererInitializationError> {
        let frame_mgr = FrameManager::new(device.clone(), command_pool, swapchain.image_len)?;

        let light_proj = glam::Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
        let light_view = glam::Mat4::look_at_rh(
            glam::Vec3::new(5.0, 10.0, 5.0), // light position
            glam::Vec3::ZERO,                // target
            glam::Vec3::Y,                   // up
        );
        let light_view_proj = light_proj * light_view;

        let shadow_ubo = ShadowUBO {
            model: glam::Mat4::IDENTITY,
            light_view_proj,
        };

        let shadow_pass =
            TestShadowPass::new(physical_device, device.clone(), render_area, shadow_ubo)?;

        let debug_quad_pass = DebugQuadPass::new(
            device.clone(),
            &swapchain,
            render_area,
            shadow_pass.depth_attachment.view,
        )?;

        Ok(Self {
            device,
            swapchain,
            frame_mgr,
            shadow_pass,
            debug_quad_pass,
        })
    }

    pub fn render(&mut self, meshes: &[MeshBuffer]) -> Result<bool, RenderError> {
        let (frame, _) = self.frame_mgr.next_frame();
        let (image_available, in_flight, cmd_buffer) =
            { (frame.image_available, frame.in_flight, frame.cmd_buffer) };

        // 이전 프레임 GPU 작업 완료 대기
        self.device.wait_for_fence(in_flight)?;
        self.device.reset_fence(in_flight)?;

        let (swapchain_image_idx, is_suboptimal) = self.swapchain.acquire_next_image(
            image_available, // wait
        )?;

        if is_suboptimal {
            log::debug!("Swapchain is suboptimal when acquire next image");
            return Ok(true);
        }

        // 이미지 전용 세마포어 가져오기
        let img = self.frame_mgr.swapchain_image(swapchain_image_idx as usize);

        self.device.reset_command_buffer(cmd_buffer)?;
        self.device.begin_command_buffer(cmd_buffer)?;

        self.shadow_pass.record_commands(cmd_buffer, meshes);

        /*transition_image_layout(
            &self.device,
            cmd_buffer,
            self.shadow_pass.depth_attachment.image,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::ImageAspectFlags::DEPTH,
        );*/

        self.debug_quad_pass
            .record_commands(cmd_buffer, swapchain_image_idx as usize);

        self.device.end_command_buffer(cmd_buffer)?;

        self.device.submit_graphics_commands(
            cmd_buffer,
            image_available,
            img.render_finished,
            in_flight,
        )?;

        let is_suboptimal =
            self.device
                .present(&self.swapchain, swapchain_image_idx, img.render_finished)?;

        if is_suboptimal {
            log::debug!("Swapchain is suboptimal when present");
        }

        Ok(is_suboptimal)
    }
}
