use std::sync::Arc;

use glam::{Vec2, Vec3};

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        BufferWithMemoryCreationError, CopyCommandBufferError, Device,
        GraphicsPipelineCreationError, MapMemoryError, MemoryUploadSlice,
        PipelineLayoutCreationError,
    },
    pipeline::graphics::GraphicsPipeline,
};
use thiserror::Error;

use crate::test_vertex_input::vertex::Vertex;

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.frag.spv");

const TEST_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: Vec2::new(-0.5, -0.5),
        color: Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, -0.5),
        color: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, 0.5),
        color: Vec3::new(0.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec2::new(-0.5, 0.5),
        color: Vec3::new(1.0, 1.0, 1.0),
    },
];

const TEST_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Debug, Error)]
pub enum BufferCreationError {
    #[error("Failed to create buffer with memory: {0}")]
    CreateBufferWithMemory(#[from] BufferWithMemoryCreationError),

    #[error("Failed to upload data to memory: {0}")]
    UploadDataToMemory(#[from] MapMemoryError),

    #[error("Failed to copy buffer: {0}")]
    CopyBuffer(#[from] CopyCommandBufferError),
}

pub struct CombinedBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub vertex_offset: vk::DeviceSize,
    pub index_offset: vk::DeviceSize,
    pub index_count: u32,
}

pub fn create_combined_buffer(
    device: &Device,
    command_pool: &CommandPool,
    vertices: &[Vertex],
    indices: &[u16],
) -> Result<CombinedBuffer, BufferCreationError> {
    let vertex_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
    let index_size = (std::mem::size_of::<u16>() * indices.len()) as vk::DeviceSize;

    let index_offset = (vertex_size + 3) & !3;
    let total_size = index_offset + index_size;

    let (staging_buffer, staging_memory) = device.create_buffer_with_memory(
        total_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * std::mem::size_of::<u16>(),
        )
    };

    let slices = [
        MemoryUploadSlice {
            src: vertex_bytes,
            dst_offset: 0,
        },
        MemoryUploadSlice {
            src: index_bytes,
            dst_offset: index_offset,
        },
    ];

    device.upload_slices_to_memory(staging_memory, total_size, &slices)?;

    let (buffer, memory) = device.create_buffer_with_memory(
        total_size,
        vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::VERTEX_BUFFER
            | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    command_pool.copy_buffer(staging_buffer, buffer, total_size)?;
    device.destroy_buffer_with_memory(staging_buffer, staging_memory);

    Ok(CombinedBuffer {
        buffer,
        memory,
        vertex_offset: 0,
        index_offset,
        index_count: indices.len() as u32,
    })
}

pub struct TestSubpass {
    device: Arc<Device>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: GraphicsPipeline,
    combined_buffer: CombinedBuffer,
}

#[derive(Debug, Error)]
pub enum TestSubpassInitializationError {
    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),

    #[error("Failed to create buffer: {0}")]
    CreateBuffer(#[from] BufferCreationError),
}

impl TestSubpass {
    pub fn new(
        device: Arc<Device>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
        render_pass: vk::RenderPass,
        subpass_index: u32,
    ) -> Result<Self, TestSubpassInitializationError> {
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

        let combined_buffer =
            create_combined_buffer(&device, command_pool, &TEST_VERTICES, &TEST_INDICES)?;

        Ok(Self {
            device,
            pipeline_layout,
            pipeline,
            combined_buffer,
        })
    }

    pub fn record_commands(&self, command_buffer: vk::CommandBuffer) {
        self.pipeline.bind_pipeline(command_buffer);

        self.device.bind_vertex_buffers(
            command_buffer,
            &[self.combined_buffer.buffer],
            &[self.combined_buffer.vertex_offset],
        );

        self.device.bind_index_buffer(
            command_buffer,
            self.combined_buffer.buffer,
            vk::IndexType::UINT16,
            self.combined_buffer.index_offset,
        );

        self.device
            .draw_indexed(command_buffer, self.combined_buffer.index_count, 1, 0, 0, 0);
    }
}

impl Drop for TestSubpass {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.device.destroy_pipeline_layout(self.pipeline_layout);
        self.device
            .destroy_buffer_with_memory(self.combined_buffer.buffer, self.combined_buffer.memory);
    }
}
