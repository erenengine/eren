use std::sync::Arc;

use glam::{Vec2, Vec3};

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        BufferWithMemoryCreationError, CopyCommandBufferError, DescriptorPoolCreationError,
        DescriptorSetAllocationError, DescriptorSetLayoutCreationError, Device,
        GraphicsPipelineCreationError, MapMemoryError, MemoryUploadSlice,
        PipelineLayoutCreationError,
    },
};
use thiserror::Error;

use super::vertex::Vertex;

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.frag.spv");

const TEST_VERTICES: [Vertex; 6] = [
    // 첫 번째 삼각형 (좌상단, 우상단, 우하단)
    Vertex {
        pos: Vec2::new(-0.5, 0.5),
        color: Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, 0.5),
        color: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, -0.5),
        color: Vec3::new(0.0, 0.0, 1.0),
    },
    // 두 번째 삼각형 (우하단, 좌하단, 좌상단)
    Vertex {
        pos: Vec2::new(0.5, -0.5),
        color: Vec3::new(0.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec2::new(-0.5, -0.5),
        color: Vec3::new(1.0, 1.0, 1.0),
    },
    Vertex {
        pos: Vec2::new(-0.5, 0.5),
        color: Vec3::new(1.0, 0.0, 0.0),
    },
];

#[derive(Debug, Error)]
pub enum BufferCreationError {
    #[error("Failed to create buffer with memory: {0}")]
    CreateBufferWithMemory(#[from] BufferWithMemoryCreationError),

    #[error("Failed to upload data to memory: {0}")]
    UploadDataToMemory(#[from] MapMemoryError),

    #[error("Failed to copy buffer: {0}")]
    CopyBuffer(#[from] CopyCommandBufferError),
}

pub fn create_vertex_buffer(
    device: &Device,
    command_pool: &CommandPool,
) -> Result<(vk::Buffer, vk::DeviceMemory), BufferCreationError> {
    let vertex_size = (std::mem::size_of::<Vertex>() * TEST_VERTICES.len()) as vk::DeviceSize;

    let (staging_buffer, staging_memory) = device.create_buffer_with_memory(
        vertex_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_VERTICES.as_ptr() as *const u8,
            TEST_VERTICES.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let slices = [MemoryUploadSlice {
        src: vertex_bytes,
        dst_offset: 0,
    }];

    device.upload_slices_to_memory(&slices, vertex_size, staging_memory)?;

    let (buffer, memory) = device.create_buffer_with_memory(
        vertex_size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    command_pool.copy_buffer(staging_buffer, buffer, vertex_size)?;
    device.destroy_buffer_with_memory(staging_buffer, staging_memory);

    Ok((buffer, memory))
}

pub struct TestSubpass {
    device: Arc<Device>,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
}

#[derive(Debug, Error)]
pub enum TestSubpassInitializationError {
    #[error("Failed to create descriptor set layout: {0}")]
    CreateDescriptorSetLayout(#[from] DescriptorSetLayoutCreationError),

    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),

    #[error("Failed to create buffer: {0}")]
    CreateBuffer(#[from] BufferCreationError),

    #[error("Failed to create buffer with memory: {0}")]
    CreateBufferWithMemory(#[from] BufferWithMemoryCreationError),

    #[error("Failed to map memory: {0}")]
    MapMemory(#[from] MapMemoryError),

    #[error("Failed to create descriptor pool: {0}")]
    CreateDescriptorPool(#[from] DescriptorPoolCreationError),

    #[error("Failed to allocate descriptor sets: {0}")]
    AllocateDescriptorSets(#[from] DescriptorSetAllocationError),
}

impl TestSubpass {
    pub fn new(
        device: Arc<Device>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
        render_pass: vk::RenderPass,
        subpass_index: u32,
    ) -> Result<Self, TestSubpassInitializationError> {
        let pipeline_layout = device.create_pipeline_layout(&[], &[])?;

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
            .subpass(subpass_index);

        let pipeline = device.create_graphics_pipeline(
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )?;

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(&device, command_pool)?;

        Ok(Self {
            device,

            pipeline_layout,
            pipeline,

            vertex_buffer,
            vertex_buffer_memory,
        })
    }

    pub fn record_commands(&mut self, command_buffer: vk::CommandBuffer) {
        self.device
            .bind_graphics_pipeline(command_buffer, self.pipeline);

        self.device
            .bind_vertex_buffers(command_buffer, &[self.vertex_buffer], &[0]);

        self.device
            .draw(command_buffer, TEST_VERTICES.len() as u32, 1, 0, 0);
    }
}

impl Drop for TestSubpass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device
            .destroy_buffer_with_memory(self.vertex_buffer, self.vertex_buffer_memory);
        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
    }
}
