use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    device::{
        DescriptorPoolCreationError, DescriptorSetAllocationError,
        DescriptorSetLayoutCreationError, Device, FramebufferCreationError,
        GraphicsPipelineCreationError, PipelineLayoutCreationError, RenderPassCreationError,
        SamplerCreationError,
    },
    swapchain::Swapchain,
};
use thiserror::Error;

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/debug_quad.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("./shaders/debug_quad.frag.spv");

const CLEAR_VALUES: [vk::ClearValue; 1] = [vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.1921, 0.302, 0.4745, 1.0],
    },
}];

pub struct DebugQuadPass {
    device: Arc<Device>,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,

    render_area: vk::Rect2D,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    shadow_map_sampler: vk::Sampler,
}

#[derive(Debug, Error)]
pub enum DebugQuadPassInitializationError {
    #[error("Failed to create descriptor set layout: {0}")]
    CreateDescriptorSetLayout(#[from] DescriptorSetLayoutCreationError),

    #[error("Failed to create descriptor pool: {0}")]
    CreateDescriptorPool(#[from] DescriptorPoolCreationError),

    #[error("Failed to allocate descriptor sets: {0}")]
    AllocateDescriptorSets(#[from] DescriptorSetAllocationError),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] RenderPassCreationError),

    #[error("Failed to create framebuffers: {0}")]
    CreateFramebuffers(#[from] FramebufferCreationError),

    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),

    #[error("Failed to create sampler: {0}")]
    CreateSampler(#[from] SamplerCreationError),
}

impl DebugQuadPass {
    pub fn new(
        device: Arc<Device>,
        swapchain: &Swapchain,
        render_area: vk::Rect2D,
        shadow_map_view: vk::ImageView,
    ) -> Result<Self, DebugQuadPassInitializationError> {
        let sampler_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout = device.create_descriptor_set_layout(&[sampler_binding])?;

        let descriptor_pool = device.create_descriptor_pool(
            1, // max sets
            &[vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
            }], // pool sizes
        )?;

        let descriptor_set =
            device.allocate_descriptor_sets(descriptor_pool, &[descriptor_set_layout])?[0];

        let pipeline_layout = device.create_pipeline_layout(&[descriptor_set_layout], &[])?;

        let color_attachment = device.get_swapchain_color_attachment_desc();
        let color_attachment_ref = device.get_color_attachment_ref(0);
        let color_refs = [color_attachment_ref];

        let subpass = vk::SubpassDescription2::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs);

        let render_pass = device.create_render_pass(&[color_attachment], &[subpass], &[])?;

        let swapchain_framebuffers = swapchain.create_framebuffers(render_pass)?;

        let binding_descriptions = [];
        let attribute_descriptions = [];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: render_area.extent.width as f32,
            height: render_area.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let viewports = [viewport];

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: render_area.extent,
        };

        let scissors = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0) // Optional
            .depth_bias_clamp(0.0) // Optional
            .depth_bias_slope_factor(0.0); // Optional

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1.0) // Optional
            .sample_mask(&[]) // Optional
            .alpha_to_coverage_enable(false) // Optional
            .alpha_to_one_enable(false); // Optional

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::ONE) // Optional
            .dst_color_blend_factor(vk::BlendFactor::ZERO) // Optional
            .color_blend_op(vk::BlendOp::ADD) // Optional
            .src_alpha_blend_factor(vk::BlendFactor::ONE) // Optional
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO) // Optional
            .alpha_blend_op(vk::BlendOp::ADD); // Optional

        let color_blend_attachment_states = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // Optional
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]); // Optional

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = device.create_graphics_pipeline(
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )?;

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_enable(false);

        let shadow_map_sampler = device.create_sampler(&sampler_info)?;

        let image_info = vk::DescriptorImageInfo {
            sampler: shadow_map_sampler, // vk::Sampler
            image_view: shadow_map_view, // vk::ImageView (depth)
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        device.write_descriptor_sets(&[write]);

        Ok(Self {
            device,

            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,

            render_area,
            render_pass,
            swapchain_framebuffers,

            pipeline_layout,
            pipeline,

            shadow_map_sampler,
        })
    }

    pub fn record_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        swapchain_image_idx: usize,
    ) {
        self.device.begin_render_pass(
            command_buffer,
            self.render_pass,
            self.swapchain_framebuffers[swapchain_image_idx],
            self.render_area,
            &CLEAR_VALUES,
        );

        self.device
            .bind_graphics_pipeline(command_buffer, self.pipeline);

        self.device.bind_graphics_descriptor_sets(
            command_buffer,
            self.pipeline_layout,
            &[self.descriptor_set], // 제공되어야 함
        );

        self.device.draw(command_buffer, 3, 1, 0, 0);

        self.device.end_render_pass(command_buffer);
    }
}

impl Drop for DebugQuadPass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device.destroy_sampler(self.shadow_map_sampler);

        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
        self.device.destroy_descriptor_pool(self.descriptor_pool);

        for &framebuffer in self.swapchain_framebuffers.iter() {
            self.device.destroy_framebuffer(framebuffer);
        }

        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);

        self.device.destroy_render_pass(self.render_pass);
    }
}
