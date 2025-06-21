use std::sync::Arc;

use ash::vk::CommandBuffer;
use eren_vulkan_render_shared::device::Device;

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texcoord: [f32; 2],
}

pub struct TestRenderView {}

pub struct TestRenderer {}

impl TestRenderer {
    pub fn new(device: Arc<Device>) -> Self {
        Self {}
    }

    pub fn render(&self, command_buffer: &mut CommandBuffer, views: &[TestRenderView]) {}
}

pub mod render_passes;
