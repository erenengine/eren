use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::device::Device;

use crate::test_render_pass::renderer::render_passes::test::{
    depth::TestDepthSubpass, main::TestMainSubpass,
};

pub mod depth;
pub mod main;

pub struct TestRenderPass {
    device: Arc<Device>,
    render_pass: vk::RenderPass,

    depth_subpass: TestDepthSubpass,
    main_subpass: TestMainSubpass,
}
