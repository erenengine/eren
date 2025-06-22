use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    device::{Device, GraphicsPipelineCreationError, PipelineLayoutCreationError},
    pipeline::graphics::GraphicsPipeline,
};
use thiserror::Error;

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../../../shaders/shader.frag.spv");

pub struct TestSubpass {
    device: Arc<Device>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: GraphicsPipeline,
}

#[derive(Debug, Error)]
pub enum TestSubpassInitializationError {
    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),
}

impl TestSubpass {
    pub fn new(
        device: Arc<Device>,
        render_area: vk::Rect2D,
        render_pass: vk::RenderPass,
        subpass_index: u32,
    ) -> Result<Self, TestSubpassInitializationError> {
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&[])
            .vertex_attribute_descriptions(&[]);

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
            .front_face(vk::FrontFace::CLOCKWISE)
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

        let pipeline_layout = device.create_pipeline_layout(&[], &[])?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(subpass_index);

        let pipeline = GraphicsPipeline::new(
            device.clone(),
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )?;

        Ok(Self {
            device,
            pipeline_layout,
            pipeline,
        })
    }

    pub fn record_commands(&self, command_buffer: vk::CommandBuffer) {
        self.pipeline.bind_pipeline(command_buffer);
        self.device.draw(command_buffer, 3, 1, 0, 0);
    }
}

impl Drop for TestSubpass {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_pipeline_layout(self.pipeline_layout);
    }
}
