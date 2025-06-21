use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::device::Device;

use crate::renderer::render_passes::scene::{
    composite::CompositeSubpass, depth::DepthSubpass, gbuffer::GBufferSubpass,
    lighting::LightingSubpass, translucent::TranslucentSubpass,
};

pub mod composite;
pub mod depth;
pub mod gbuffer;
pub mod lighting;
pub mod translucent;

pub struct SceneRenderPass {
    device: Arc<Device>,
    render_pass: vk::RenderPass,

    depth_subpass: DepthSubpass,
    gbuffer_subpass: GBufferSubpass,
    lighting_subpass: LightingSubpass,
    translucent_subpass: TranslucentSubpass,
    composite_subpass: CompositeSubpass,
}
