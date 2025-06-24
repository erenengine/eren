use std::sync::Arc;

use ash::vk;

use crate::device::{CommandBufferAllocationError, CommandPoolCreationError, Device};

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
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_command_pool(self.handle);
    }
}
