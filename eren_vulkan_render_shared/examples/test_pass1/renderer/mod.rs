use std::sync::Arc;

use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{Device, SubmitGraphicsCommandsError, WaitForFencesError},
    frame::{FrameManager, FrameManagerInitializationError, NextFrameError},
    swapchain::{Swapchain, SwapchainAcquireError, SwapchainPresentError},
};
use thiserror::Error;

use crate::test_pass1::renderer::render_passes::test::TestRenderPassInitializationError;

pub mod render_passes;

pub struct TestRenderer {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    frame_mgr: FrameManager,
    render_pass: render_passes::test::TestRenderPass,
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
    #[error("Failed to get next frame: {0}")]
    NextFrame(#[from] NextFrameError),

    #[error("Failed to wait for fences: {0}")]
    WaitForFences(#[from] WaitForFencesError),

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
        command_pool: Arc<CommandPool>,
    ) -> Result<Self, TestRendererInitializationError> {
        let frame_mgr = FrameManager::new(device.clone(), command_pool.clone())?;
        let render_pass = render_passes::test::TestRenderPass::new(device.clone())?;

        Ok(Self {
            device,
            swapchain,
            frame_mgr,
            render_pass,
        })
    }

    pub fn render(&mut self) -> Result<(), RenderError> {
        let frame = self.frame_mgr.next()?;

        let (image_idx, is_suboptimal) =
            self.swapchain.acquire_next_image(frame.image_available)?;

        //TODO: handle suboptimal

        self.render_pass
            .record_commands(frame.cmd_buffer, image_idx);

        /*self.device.submit_graphics_commands(
            frame.cmd_buffer,
            frame.image_available,
            frame.render_finished,
            frame.in_flight,
        )?;

        let is_suboptimal =
            self.device
                .present(&self.swapchain, image_idx, frame.render_finished)?;*/

        //TODO: handle suboptimal

        Ok(())
    }
}
