use std::sync::Arc;

use eren_vulkan_render_shared::device::{Device, ShaderModuleCreationError};

pub struct TestSubpass {}

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.frag.spv");

impl TestSubpass {
    pub fn new(device: Arc<Device>) -> Result<Self, ShaderModuleCreationError> {
        let vertex_shader_module = device.create_shader_module(VERT_SHADER_BYTES)?;
        let fragment_shader_module = device.create_shader_module(FRAG_SHADER_BYTES)?;

        log::debug!("Vertex shader module created");
        log::debug!("Fragment shader module created");

        //TODO: create pipeline

        device.destroy_shader_module(vertex_shader_module);
        device.destroy_shader_module(fragment_shader_module);

        Ok(Self {})
    }
}
