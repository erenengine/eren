use std::sync::Arc;

use ash::vk;

use crate::device::{Device, GraphicsPipelineCreationError};

pub struct GraphicsPipeline {
    device: Arc<Device>,
    handle: vk::Pipeline,
}

impl GraphicsPipeline {
    pub fn new(
        device: Arc<Device>,
        info: vk::GraphicsPipelineCreateInfo,
        vert_shader_bytes: Option<&'static [u8]>,
        frag_shader_bytes: Option<&'static [u8]>,
    ) -> Result<Self, GraphicsPipelineCreationError> {
        let handle = device.create_graphics_pipeline(info, vert_shader_bytes, frag_shader_bytes)?;
        Ok(Self { device, handle })
    }

    pub fn bind_pipeline(&self, command_buffer: vk::CommandBuffer) {
        self.device
            .bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.handle);
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_pipeline(self.handle);
    }
}
