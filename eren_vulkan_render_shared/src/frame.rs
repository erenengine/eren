use std::sync::Arc;

use crate::{
    command::CommandPool,
    device::{CommandBufferAllocationError, Device, FenceCreationError, SemaphoreCreationError},
};
use ash::vk;
use thiserror::Error;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    pub image_available: vk::Semaphore,
    pub in_flight: vk::Fence,
    pub cmd_buffer: vk::CommandBuffer,
}

pub struct SwapchainImageData {
    pub render_finished: vk::Semaphore,
}

pub struct FrameManager {
    device: Arc<Device>,
    frames: Vec<FrameData>,
    swapchain_images: Vec<SwapchainImageData>,
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

impl FrameManager {
    pub fn new(
        device: Arc<Device>,
        command_pool: &CommandPool,
        swapchain_len: usize,
    ) -> Result<Self, FrameManagerInitializationError> {
        // 프레임 자원
        let cmd_buffers = command_pool.allocate_command_buffers(MAX_FRAMES_IN_FLIGHT as u32)?;
        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            frames.push(FrameData {
                image_available: device.create_semaphore()?,
                in_flight: device.create_fence_signaled()?,
                cmd_buffer: cmd_buffers[i],
            });
        }

        // 이미지 자원
        let mut swapchain_images = Vec::with_capacity(swapchain_len);
        for _ in 0..swapchain_len {
            swapchain_images.push(SwapchainImageData {
                render_finished: device.create_semaphore()?,
            });
        }

        Ok(Self {
            device,
            frames,
            swapchain_images,
            current_frame: 0,
        })
    }

    pub fn next_frame(&mut self) -> &FrameData {
        let idx = self.current_frame;
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        &self.frames[idx]
    }

    pub fn swapchain_image(&self, image_index: usize) -> &SwapchainImageData {
        &self.swapchain_images[image_index]
    }
}

impl Drop for FrameManager {
    fn drop(&mut self) {
        self.device.wait_idle();

        // 프레임 자원
        for f in &self.frames {
            self.device.destroy_semaphore(f.image_available);
            self.device.destroy_fence(f.in_flight);
        }

        // 이미지 자원
        for img in &self.swapchain_images {
            self.device.destroy_semaphore(img.render_finished);
        }
    }
}
