use std::sync::Arc;

use ash::vk;

use crate::device::{
    CommandBufferAllocationError, CommandPoolCreationError, CopyCommandBufferError, Device,
};

// 커맨드 풀은 스레드 당 하나씩 생성해야 합니다. 스레드 간 공유할 수 없습니다.
pub struct CommandPool {
    device: Arc<Device>,
    handle: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: Arc<Device>) -> Result<Self, CommandPoolCreationError> {
        let handle = device.create_command_pool()?;

        Ok(Self { device, handle })
    }

    pub fn allocate_command_buffers(
        &self,
        command_buffer_count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, CommandBufferAllocationError> {
        Ok(self
            .device
            .allocate_command_buffers(self.handle, command_buffer_count)?)
    }

    pub fn copy_buffer(
        &self,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<(), CopyCommandBufferError> {
        self.device
            .copy_command_buffer(self.handle, src_buffer, dst_buffer, size)
    }

    pub fn transition_image_layout(
        &self,
        image: vk::Image,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_stage: vk::PipelineStageFlags,
        dst_stage: vk::PipelineStageFlags,
    ) -> Result<(), CopyCommandBufferError> {
        self.device.transition_image_layout(
            self.handle,
            image,
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_stage,
            dst_stage,
        )
    }

    pub fn copy_buffer_to_image(
        &self,
        src_buffer: vk::Buffer,
        dst_image: vk::Image,
        width: u32,
        height: u32,
    ) -> Result<(), CopyCommandBufferError> {
        self.device
            .copy_buffer_to_image(self.handle, src_buffer, dst_image, width, height)
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_command_pool(self.handle);
    }
}
