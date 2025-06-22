use std::sync::Arc;

use crate::{
    command::CommandPool,
    device::{
        CommandBufferAllocationError, CommandBufferResetError, Device, FenceCreationError,
        SemaphoreCreationError, WaitForFencesError,
    },
};
use ash::vk;
use thiserror::Error;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    pub image_available: vk::Semaphore,
    pub render_finished: vk::Semaphore,
    pub in_flight: vk::Fence,
    pub cmd_buffer: vk::CommandBuffer,
}

pub struct FrameManager {
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    frames: Vec<FrameData>,
    current_frame: usize,
}

#[derive(Debug, Error)]
pub enum FrameManagerInitializationError {
    #[error("Failed to allocate command buffers: {0}")]
    AllocateCommandBuffers(#[from] CommandBufferAllocationError),

    #[error("Failed to create semaphore: {0}")]
    CreateSemaphore(#[from] SemaphoreCreationError),

    #[error("Failed to create fence: {0}")]
    CreateFence(#[from] FenceCreationError),
}

#[derive(Debug, Error)]
pub enum NextFrameError {
    #[error("Failed to wait for fences: {0}")]
    WaitForFences(#[from] WaitForFencesError),

    #[error("Failed to reset command buffer: {0}")]
    ResetCommandBuffer(#[from] CommandBufferResetError),
}

impl FrameManager {
    pub fn new(
        device: Arc<Device>,
        command_pool: Arc<CommandPool>,
    ) -> Result<Self, FrameManagerInitializationError> {
        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);

        let cmd_buffers = command_pool.allocate_command_buffers(MAX_FRAMES_IN_FLIGHT as u32)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let frame_data = FrameData {
                image_available: device.create_semaphore()?,
                render_finished: device.create_semaphore()?,
                in_flight: device.create_fence_signaled()?,
                cmd_buffer: cmd_buffers[i],
            };
            frames.push(frame_data);
        }

        Ok(Self {
            device,
            command_pool,
            frames,
            current_frame: 0,
        })
    }

    pub fn next(&mut self) -> Result<&mut FrameData, NextFrameError> {
        let frame = &mut self.frames[self.current_frame];

        //self.device.wait_for_fence(frame.in_flight)?;
        //self.device.reset_command_buffer(frame.cmd_buffer)?;

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(frame)
    }
}

impl Drop for FrameManager {
    fn drop(&mut self) {
        self.device.wait_idle();

        for frame_data in &self.frames {
            self.device.destroy_semaphore(frame_data.image_available);
            self.device.destroy_semaphore(frame_data.render_finished);
            self.device.destroy_fence(frame_data.in_flight);
        }
    }
}
