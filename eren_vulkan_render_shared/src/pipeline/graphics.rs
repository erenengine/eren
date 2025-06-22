use std::sync::Arc;

use ash::vk;

use crate::device::{Device, GraphicsPipelineCreationError};

pub struct GraphicsPipelineConfig {
    pub vert_shader_bytes: Option<&'static [u8]>,
    pub frag_shader_bytes: Option<&'static [u8]>,
    pub render_pass_handle: vk::RenderPass,
    pub subpass_index: u32,
}

pub struct GraphicsPipeline {
    device: Arc<Device>,
    handle: vk::Pipeline,
}

impl GraphicsPipeline {
    pub fn new(
        device: Arc<Device>,
        config: GraphicsPipelineConfig,
    ) -> Result<Self, GraphicsPipelineCreationError> {
        let info = vk::GraphicsPipelineCreateInfo::default()
            .render_pass(config.render_pass_handle)
            .subpass(config.subpass_index);

        let handle = device.create_graphics_pipeline(
            info,
            config.vert_shader_bytes,
            config.frag_shader_bytes,
        )?;

        Ok(Self { device, handle })
    }
}

impl Drop for GraphicsPipeline {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_pipeline(self.handle);
    }
}
