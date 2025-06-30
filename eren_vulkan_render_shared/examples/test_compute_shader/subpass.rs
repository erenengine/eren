use std::sync::Arc;

use glam::{Vec2, Vec3};

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        BufferWithMemoryCreationError, ComputePipelineCreationError, CopyCommandBufferError,
        DescriptorPoolCreationError, DescriptorSetAllocationError,
        DescriptorSetLayoutCreationError, Device, GraphicsPipelineCreationError, MapMemoryError,
        MemoryUploadSlice, PipelineLayoutCreationError,
    },
    frame::MAX_FRAMES_IN_FLIGHT,
};
use thiserror::Error;

use super::{push_constants::ComputePushConstants, ssbo::StorageBufferObject, vertex::Vertex};

const COMP_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.comp.spv");
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
) -> Result<CombinedBuffer, BufferCreationError> {
    let vertex_size = (std::mem::size_of::<Vertex>() * TEST_VERTICES.len()) as vk::DeviceSize;
    let index_size = (std::mem::size_of::<u16>() * TEST_INDICES.len()) as vk::DeviceSize;

    let index_offset = (vertex_size + 3) & !3;
    let total_size = index_offset + index_size;

    let (staging_buffer, staging_memory) = device.create_buffer_with_memory(
        total_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_VERTICES.as_ptr() as *const u8,
            TEST_VERTICES.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_INDICES.as_ptr() as *const u8,
            TEST_INDICES.len() * std::mem::size_of::<u16>(),
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
        index_count: TEST_INDICES.len() as u32,
    })
}

pub struct TestSubpass {
    device: Arc<Device>,

    descriptor_set_layout: vk::DescriptorSetLayout,

    compute_pipeline_layout: vk::PipelineLayout,
    compute_pipeline: vk::Pipeline,

    graphics_pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,

    combined_buffer: CombinedBuffer,
    storage_buffers: Vec<vk::Buffer>,
    storage_buffers_memory: Vec<vk::DeviceMemory>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    start_time: std::time::Instant,
}

#[derive(Debug, Error)]
pub enum TestSubpassInitializationError {
    #[error("Failed to create descriptor set layout: {0}")]
    CreateDescriptorSetLayout(#[from] DescriptorSetLayoutCreationError),

    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create compute pipeline: {0}")]
    CreateComputePipeline(#[from] ComputePipelineCreationError),

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
        let ssbo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX);

        let descriptor_set_layout = device.create_descriptor_set_layout(&[ssbo_layout_binding])?;

        // push constant range 정의
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(std::mem::size_of::<ComputePushConstants>() as u32);

        // compute pipeline layout: 기존 descriptor_set_layout 재사용
        let compute_pipeline_layout =
            device.create_pipeline_layout(&[descriptor_set_layout], &[push_constant_range])?;

        // 컴퓨트 파이프라인 생성
        let create_info = vk::ComputePipelineCreateInfo::default().layout(compute_pipeline_layout);
        let compute_pipeline = device.create_compute_pipeline(create_info, COMP_SHADER_BYTES)?;

        let graphics_pipeline_layout =
            device.create_pipeline_layout(&[descriptor_set_layout], &[])?;

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
            .layout(graphics_pipeline_layout)
            .render_pass(render_pass)
            .subpass(subpass_index);

        let graphics_pipeline = device.create_graphics_pipeline(
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )?;

        let combined_buffer = create_combined_buffer(&device, command_pool)?;

        let buffer_size = std::mem::size_of::<StorageBufferObject>() as vk::DeviceSize;

        let mut storage_buffers = Vec::new();
        let mut storage_buffers_memory = Vec::new();

        storage_buffers.resize(MAX_FRAMES_IN_FLIGHT, vk::Buffer::null());
        storage_buffers_memory.resize(MAX_FRAMES_IN_FLIGHT, vk::DeviceMemory::null());

        let descriptor_pool = device.create_descriptor_pool(
            MAX_FRAMES_IN_FLIGHT as u32,
            &[vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
            }],
        )?;

        let descriptor_set_layouts = vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let descriptor_sets =
            device.allocate_descriptor_sets(descriptor_pool, &descriptor_set_layouts)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let (buffer, memory) = device.create_buffer_with_memory(
                buffer_size,
                vk::BufferUsageFlags::STORAGE_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;

            storage_buffers[i] = buffer;
            storage_buffers_memory[i] = memory;

            let buffer_info = vk::DescriptorBufferInfo {
                buffer,
                offset: 0,
                range: std::mem::size_of::<StorageBufferObject>() as vk::DeviceSize,
            };

            let buffer_infos = [buffer_info];
            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_infos);

            device.write_descriptor_sets(&[descriptor_write]);
        }

        Ok(Self {
            device,

            descriptor_set_layout,

            compute_pipeline_layout,
            compute_pipeline,

            graphics_pipeline_layout,
            graphics_pipeline,

            combined_buffer,
            storage_buffers,
            storage_buffers_memory,
            descriptor_pool,
            descriptor_sets,

            start_time: std::time::Instant::now(),
        })
    }

    pub fn record_compute_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_idx: usize,
        extent: vk::Extent2D,
        pre_transform: vk::SurfaceTransformFlagsKHR,
    ) {
        let time = self.start_time.elapsed().as_secs_f32();
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let push_constants = ComputePushConstants {
            time,
            aspect_ratio,
            pre_transform: pre_transform.as_raw(),
            _padding: 0,
        };

        // compute 바인딩
        self.device
            .bind_compute_pipeline(command_buffer, self.compute_pipeline);

        let push_constants_bytes = unsafe {
            std::slice::from_raw_parts(
                &push_constants as *const ComputePushConstants as *const u8,
                std::mem::size_of::<ComputePushConstants>(),
            )
        };

        // push constants
        self.device.push_constants(
            command_buffer,
            self.compute_pipeline_layout,
            vk::ShaderStageFlags::COMPUTE,
            0,
            push_constants_bytes,
        );

        // descriptor set
        self.device.bind_compute_descriptor_sets(
            command_buffer,
            self.compute_pipeline_layout,
            &[self.descriptor_sets[frame_idx]],
        );

        // dispatch
        self.device.dispatch(command_buffer, 1, 1, 1);
    }

    pub fn record_graphics_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_idx: usize,
    ) {
        self.device
            .bind_graphics_pipeline(command_buffer, self.graphics_pipeline);

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

        self.device.bind_graphics_descriptor_sets(
            command_buffer,
            self.graphics_pipeline_layout,
            &[self.descriptor_sets[frame_idx]],
        );

        self.device
            .draw_indexed(command_buffer, self.combined_buffer.index_count, 1, 0, 0, 0);
    }
}

impl Drop for TestSubpass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device.destroy_descriptor_pool(self.descriptor_pool);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            self.device.destroy_buffer_with_memory(
                self.storage_buffers[i],
                self.storage_buffers_memory[i],
            );
        }

        self.device
            .destroy_buffer_with_memory(self.combined_buffer.buffer, self.combined_buffer.memory);

        self.device.destroy_pipeline(self.compute_pipeline);
        self.device
            .destroy_pipeline_layout(self.compute_pipeline_layout);

        self.device.destroy_pipeline(self.graphics_pipeline);
        self.device
            .destroy_pipeline_layout(self.graphics_pipeline_layout);

        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
    }
}
