use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    device::{Device, GraphicsPipelineCreationError},
    pipeline::graphics::{GraphicsPipeline, GraphicsPipelineConfig},
};

pub struct TestSubpass {
    pipeline: GraphicsPipeline,
}

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.frag.spv");

impl TestSubpass {
    pub fn new(
        device: Arc<Device>,
        render_pass_handle: vk::RenderPass,
        subpass_index: u32,
    ) -> Result<Self, GraphicsPipelineCreationError> {
        let pipeline = GraphicsPipeline::new(
            device.clone(),
            GraphicsPipelineConfig {
                vert_shader_bytes: Some(VERT_SHADER_BYTES),
                frag_shader_bytes: Some(FRAG_SHADER_BYTES),
                render_pass_handle,
                subpass_index,
            },
        )?;

        Ok(Self { pipeline })
    }

    pub fn record_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        current_swapchain_framebuffer: vk::Framebuffer,
    ) {
    }
}
