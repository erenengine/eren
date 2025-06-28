use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        CommandBufferBeginError, CommandBufferEndError, CommandBufferResetError, Device,
        ResetFencesError, SubmitGraphicsCommandsError, WaitForFencesError,
    },
    frame::{FrameManager, FrameManagerInitializationError},
    swapchain::{Swapchain, SwapchainAcquireError, SwapchainPresentError},
};
use thiserror::Error;

use crate::test_vertex_input::render_pass::{TestRenderPass, TestRenderPassInitializationError};

pub struct TestRenderer {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    frame_mgr: FrameManager,
    render_pass: TestRenderPass,
}

#[derive(Debug, Error)]
pub enum TestRendererInitializationError {
    #[error("Failed to create frame manager: {0}")]
    CreateFrameManager(#[from] FrameManagerInitializationError),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] TestRenderPassInitializationError),
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

impl TestRenderer {
    pub fn new(
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestRendererInitializationError> {
        let frame_mgr = FrameManager::new(device.clone(), command_pool, swapchain.image_len)?;
        let render_pass =
            TestRenderPass::new(device.clone(), &swapchain, command_pool, render_area)?;

        Ok(Self {
            device,
            swapchain,
            frame_mgr,
            render_pass,
        })
    }

    pub fn render(&mut self) -> Result<bool, RenderError> {
        let (frame, frame_idx) = self.frame_mgr.next_frame();
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

        self.render_pass.record_commands(
            cmd_buffer,
            swapchain_image_idx as usize,
            frame_idx,
            self.swapchain.window_width,
            self.swapchain.window_height,
            self.swapchain.pre_transform,
        );

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
