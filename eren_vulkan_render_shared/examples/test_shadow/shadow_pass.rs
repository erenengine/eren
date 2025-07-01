use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    attachment::Attachment,
    device::{
        AttachmentCreationError, BufferWithMemoryCreationError, DescriptorPoolCreationError,
        DescriptorSetAllocationError, DescriptorSetLayoutCreationError, Device,
        FramebufferCreationError, GraphicsPipelineCreationError, MapMemoryError,
        PipelineLayoutCreationError, RenderPassCreationError,
    },
    physical_device::PhysicalDevice,
};
use thiserror::Error;

use crate::test_shadow::{
    mesh::{MeshBuffer, Vertex},
    ubo::ShadowUBO,
};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shadow.vert.spv");

const CLEAR_VALUES: [vk::ClearValue; 1] = [vk::ClearValue {
    depth_stencil: vk::ClearDepthStencilValue {
        depth: 1.0,
        stencil: 0,
    },
}];

pub struct TestShadowPass {
    device: Arc<Device>,

    pub depth_attachment: Attachment,

    render_area: vk::Rect2D,
    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,

    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,

    uniform_buffer: vk::Buffer,
    uniform_buffer_memory: vk::DeviceMemory,
    uniform_buffer_mapped: *mut std::ffi::c_void,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

#[derive(Debug, Error)]
pub enum TestShadowPassInitializationError {
    #[error("Failed to create depth attachment: {0}")]
    CreateDepthAttachment(#[from] AttachmentCreationError),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] RenderPassCreationError),

    #[error("Failed to create framebuffer: {0}")]
    CreateFramebuffer(#[from] FramebufferCreationError),

    #[error("Failed to create descriptor set layout: {0}")]
    CreateDescriptorSetLayout(#[from] DescriptorSetLayoutCreationError),

    #[error("Failed to create descriptor pool: {0}")]
    CreateDescriptorPool(#[from] DescriptorPoolCreationError),

    #[error("Failed to create descriptor sets: {0}")]
    CreateDescriptorSets(#[from] DescriptorSetAllocationError),

    #[error("Failed to create uniform buffer: {0}")]
    CreateUniformBuffer(#[from] BufferWithMemoryCreationError),

    #[error("Failed to map memory: {0}")]
    MapMemory(#[from] MapMemoryError),

    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),
}

impl TestShadowPass {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestShadowPassInitializationError> {
        let depth_attachment = device.create_depth_attachment(
            render_area.extent,
            physical_device.depth_format,
            physical_device.uses_stencil,
            vk::SampleCountFlags::TYPE_1,
            true,
        )?;

        let depth_attachment_ref = device.get_depth_attachment_ref(0, physical_device.uses_stencil);

        let subpass = vk::SubpassDescription2::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .depth_stencil_attachment(&depth_attachment_ref);

        let render_pass = device.create_render_pass(&[depth_attachment.desc], &[subpass], &[])?;

        let attachments = [depth_attachment.view];
        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(render_area.extent.width)
            .height(render_area.extent.height)
            .layers(1); // 1개의 레이어만 있는 2D 이미지 뷰

        let framebuffer = device.create_framebuffer(framebuffer_info)?;

        let descriptor_pool = device.create_descriptor_pool(
            1,
            &[vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            }],
        )?;

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let descriptor_set_layout = device.create_descriptor_set_layout(&[ubo_layout_binding])?;

        let descriptor_set_layouts = vec![descriptor_set_layout];
        let descriptor_set =
            device.allocate_descriptor_sets(descriptor_pool, &descriptor_set_layouts)?[0];

        let buffer_size = std::mem::size_of::<ShadowUBO>() as vk::DeviceSize;

        let (buffer, memory) = device.create_buffer_with_memory(
            buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let uniform_buffer = buffer;
        let uniform_buffer_memory = memory;
        let uniform_buffer_mapped = device.map_memory(uniform_buffer_memory, buffer_size)?;

        let buffer_info = vk::DescriptorBufferInfo {
            buffer,
            offset: 0,
            range: std::mem::size_of::<ShadowUBO>() as vk::DeviceSize,
        };

        let buffer_infos = [buffer_info];
        let descriptor_write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos);

        device.write_descriptor_sets(&[descriptor_write]);

        let pipeline_layout = device.create_pipeline_layout(&[descriptor_set_layout], &[])?;

        let binding_descriptions = [Vertex::get_binding_description()];
        let attribute_descriptions = Vertex::get_attribute_descriptions();

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

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // Optional
            .attachments(&[])
            .blend_constants([0.0, 0.0, 0.0, 0.0]); // Optional

        let (front, back) = if physical_device.uses_stencil {
            let default_stencil_op_state = vk::StencilOpState::default()
                .fail_op(vk::StencilOp::KEEP)
                .pass_op(vk::StencilOp::KEEP)
                .depth_fail_op(vk::StencilOp::KEEP)
                .compare_op(vk::CompareOp::ALWAYS)
                .compare_mask(0xFF)
                .write_mask(0xFF)
                .reference(0);
            (default_stencil_op_state, default_stencil_op_state)
        } else {
            // 비활성화할 경우에도 기본값을 넣어줌 (Vulkan 요구 사항)
            (vk::StencilOpState::default(), vk::StencilOpState::default())
        };

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(physical_device.uses_stencil)
            .front(front)
            .back(back);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline =
            device.create_graphics_pipeline(pipeline_info, Some(VERT_SHADER_BYTES), None)?;

        Ok(Self {
            device,

            depth_attachment,
            render_area,
            render_pass,
            framebuffer,

            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,

            uniform_buffer,
            uniform_buffer_memory,
            uniform_buffer_mapped,

            pipeline_layout,
            pipeline,
        })
    }

    pub fn update_shadow_ubo(&mut self, shadow_ubo: ShadowUBO) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                &shadow_ubo,
                self.uniform_buffer_mapped as *mut ShadowUBO,
                1,
            );
        }
    }

    pub fn record_commands(&mut self, command_buffer: vk::CommandBuffer, meshes: &[MeshBuffer]) {
        self.device.begin_render_pass(
            command_buffer,
            self.render_pass,
            self.framebuffer,
            self.render_area,
            &CLEAR_VALUES,
        );

        self.device
            .bind_graphics_pipeline(command_buffer, self.pipeline);

        self.device.bind_graphics_descriptor_sets(
            command_buffer,
            self.pipeline_layout,
            &[self.descriptor_set],
        );

        for mesh in meshes {
            self.device
                .bind_vertex_buffers(command_buffer, &[mesh.buffer], &[mesh.vertex_offset]);

            self.device.bind_index_buffer(
                command_buffer,
                mesh.buffer,
                vk::IndexType::UINT16,
                mesh.index_offset,
            );

            self.device
                .draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);
        }

        self.device.end_render_pass(command_buffer);
    }
}

impl Drop for TestShadowPass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);

        self.device
            .destroy_buffer_with_memory(self.uniform_buffer, self.uniform_buffer_memory);
        self.device.destroy_descriptor_pool(self.descriptor_pool);
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);

        self.device.destroy_framebuffer(self.framebuffer);
        self.device.destroy_render_pass(self.render_pass);
        self.device.destroy_attachment(&self.depth_attachment);
    }
}
