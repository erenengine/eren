use std::sync::Arc;

use glam::{Vec2, Vec3};

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        BufferWithMemoryCreationError, CopyCommandBufferError, DescriptorPoolCreationError,
        DescriptorSetAllocationError, DescriptorSetLayoutCreationError, Device,
        GraphicsPipelineCreationError, ImageViewCreationError, ImageWithMemoryCreationError,
        MapMemoryError, MemoryUploadSlice, PipelineLayoutCreationError, SamplerCreationError,
    },
    frame::MAX_FRAMES_IN_FLIGHT,
    physical_device::PhysicalDevice,
};
use image::{GenericImageView, ImageError};
use thiserror::Error;

use super::{ubo::UniformBufferObject, vertex::Vertex};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("./shaders/shader.frag.spv");

const TEST_VERTICES: [Vertex; 8] = [
    Vertex {
        pos: Vec3::new(-0.5, -0.5, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        tex_coords: Vec2::new(0.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(0.5, -0.5, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        tex_coords: Vec2::new(1.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(0.5, 0.5, 0.0),
        color: Vec3::new(0.0, 0.0, 1.0),
        tex_coords: Vec2::new(1.0, 1.0),
    },
    Vertex {
        pos: Vec3::new(-0.5, 0.5, 0.0),
        color: Vec3::new(1.0, 1.0, 1.0),
        tex_coords: Vec2::new(0.0, 1.0),
    },
    Vertex {
        pos: Vec3::new(-0.5, -0.5, -0.5),
        color: Vec3::new(1.0, 0.0, 0.0),
        tex_coords: Vec2::new(0.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(0.5, -0.5, -0.5),
        color: Vec3::new(0.0, 1.0, 0.0),
        tex_coords: Vec2::new(1.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(0.5, 0.5, -0.5),
        color: Vec3::new(0.0, 0.0, 1.0),
        tex_coords: Vec2::new(1.0, 1.0),
    },
    Vertex {
        pos: Vec3::new(-0.5, 0.5, -0.5),
        color: Vec3::new(1.0, 1.0, 1.0),
        tex_coords: Vec2::new(0.0, 1.0),
    },
];

const TEST_INDICES: [u16; 12] = [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

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

    device.upload_slices_to_memory(&slices, total_size, staging_memory)?;

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

    staging_buffer: vk::Buffer,
    staging_buffer_memory: vk::DeviceMemory,
    texture_image: vk::Image,
    texture_image_memory: vk::DeviceMemory,
    texture_image_view: vk::ImageView,
    texture_sampler: vk::Sampler,

    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    combined_buffer: CombinedBuffer,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    uniform_buffers_mapped: Vec<*mut std::ffi::c_void>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    start_time: std::time::Instant,
}

#[derive(Debug, Error)]
pub enum TestSubpassInitializationError {
    #[error("Failed to load image: {0}")]
    LoadImage(#[from] ImageError),

    #[error("Failed to create image with memory: {0}")]
    CreateImageWithMemory(#[from] ImageWithMemoryCreationError),

    #[error("Failed to copy buffer to image: {0}")]
    CopyBufferToImage(#[from] CopyCommandBufferError),

    #[error("Failed to create image view: {0}")]
    CreateImageView(#[from] ImageViewCreationError),

    #[error("Failed to create sampler: {0}")]
    CreateSampler(#[from] SamplerCreationError),

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
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
        render_pass: vk::RenderPass,
        subpass_index: u32,
        image_bytes: &[u8],
    ) -> Result<Self, TestSubpassInitializationError> {
        let diffuse_image = image::load_from_memory(image_bytes)?;
        let diffuse_rgba = diffuse_image.to_rgba8().into_raw();
        let dimensions = diffuse_image.dimensions();

        let image_size = (dimensions.0 * dimensions.1 * 4) as vk::DeviceSize;
        let (staging_buffer, staging_buffer_memory) = device.create_buffer_with_memory(
            image_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        device.upload_data_to_memory(&diffuse_rgba, staging_buffer_memory)?;

        let (texture_image, texture_image_memory) = device.create_image_with_memory(
            vk::Format::R8G8B8A8_SRGB,
            vk::Extent2D {
                width: dimensions.0,
                height: dimensions.1,
            },
            vk::SampleCountFlags::TYPE_1,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::ImageTiling::OPTIMAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        command_pool.transition_image_layout(
            texture_image,
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        )?;

        command_pool.copy_buffer_to_image(
            staging_buffer,
            texture_image,
            dimensions.0,
            dimensions.1,
        )?;

        command_pool.transition_image_layout(
            texture_image,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        )?;

        let texture_image_view = device.create_image_view(
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR);

        let texture_sampler = device.create_sampler(&sampler_info)?;

        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let sampler_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout =
            device.create_descriptor_set_layout(&[ubo_layout_binding, sampler_layout_binding])?;

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
            .subpass(subpass_index);

        let pipeline = device.create_graphics_pipeline(
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )?;

        let combined_buffer = create_combined_buffer(&device, command_pool)?;

        let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

        let mut uniform_buffers = Vec::new();
        let mut uniform_buffers_memory = Vec::new();
        let mut uniform_buffers_mapped = Vec::new();

        uniform_buffers.resize(MAX_FRAMES_IN_FLIGHT, vk::Buffer::null());
        uniform_buffers_memory.resize(MAX_FRAMES_IN_FLIGHT, vk::DeviceMemory::null());
        uniform_buffers_mapped.resize(MAX_FRAMES_IN_FLIGHT, std::ptr::null_mut());

        let descriptor_pool = device.create_descriptor_pool(
            MAX_FRAMES_IN_FLIGHT as u32,
            &[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
                },
            ],
        )?;

        let descriptor_set_layouts = vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];
        let descriptor_sets =
            device.allocate_descriptor_sets(descriptor_pool, &descriptor_set_layouts)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let (buffer, memory) = device.create_buffer_with_memory(
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            uniform_buffers[i] = buffer;
            uniform_buffers_memory[i] = memory;
            uniform_buffers_mapped[i] =
                device.map_memory(uniform_buffers_memory[i], buffer_size)?;

            let buffer_info = vk::DescriptorBufferInfo {
                buffer,
                offset: 0,
                range: std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
            };

            let buffer_infos = [buffer_info];

            let image_info = [vk::DescriptorImageInfo {
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                image_view: texture_image_view,
                sampler: texture_sampler,
            }];

            let descriptor_writes = [
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_infos),
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_info),
            ];

            device.write_descriptor_sets(&descriptor_writes);
        }

        Ok(Self {
            device,

            staging_buffer,
            staging_buffer_memory,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,

            descriptor_set_layout,
            pipeline_layout,
            pipeline,

            combined_buffer,
            uniform_buffers,
            uniform_buffers_memory,
            uniform_buffers_mapped,
            descriptor_pool,
            descriptor_sets,

            start_time: std::time::Instant::now(),
        })
    }

    fn update_uniform_buffer(
        &mut self,
        frame_idx: usize,
        extent: vk::Extent2D,
        pre_transform: vk::SurfaceTransformFlagsKHR,
    ) {
        let time = self.start_time.elapsed().as_secs_f32();

        // 모델 행렬: Z축 회전
        let model = glam::Mat4::from_rotation_z(-time.to_radians() * 90.0);

        // 뷰 행렬: 카메라 위치 설정
        let eye = glam::Vec3::new(2.0, 2.0, 2.0);
        let center = glam::Vec3::ZERO;
        let up = glam::Vec3::Z;
        let view = glam::Mat4::look_at_rh(eye, center, up);

        // 프로젝션 행렬
        let aspect_ratio = extent.width as f32 / extent.height as f32;
        let mut proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 10.0);

        // Vulkan에서는 Y축 뒤집기 필요
        proj.y_axis.y *= -1.0;

        // 화면 회전에 따른 프로젝션 행렬 수정
        let correction = match pre_transform {
            vk::SurfaceTransformFlagsKHR::ROTATE_90 => {
                glam::Mat4::from_rotation_z(90_f32.to_radians())
            }
            vk::SurfaceTransformFlagsKHR::ROTATE_180 => {
                glam::Mat4::from_rotation_z(180_f32.to_radians())
            }
            vk::SurfaceTransformFlagsKHR::ROTATE_270 => {
                glam::Mat4::from_rotation_z(270_f32.to_radians())
            }
            _ => glam::Mat4::IDENTITY,
        };
        proj = correction * proj;

        let ubo = UniformBufferObject { model, view, proj };

        // 메모리에 데이터 복사
        unsafe {
            let data_ptr = self.uniform_buffers_mapped[frame_idx];
            std::ptr::copy_nonoverlapping(&ubo, data_ptr as *mut UniformBufferObject, 1);
        }
    }

    pub fn record_commands(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_idx: usize,
        extent: vk::Extent2D,
        pre_transform: vk::SurfaceTransformFlagsKHR,
    ) {
        self.device
            .bind_graphics_pipeline(command_buffer, self.pipeline);

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

        self.update_uniform_buffer(frame_idx, extent, pre_transform);

        self.device.bind_graphics_descriptor_sets(
            command_buffer,
            self.pipeline_layout,
            &[self.descriptor_sets[frame_idx]],
        );

        self.device
            .draw_indexed(command_buffer, self.combined_buffer.index_count, 1, 0, 0, 0);
    }
}

impl Drop for TestSubpass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device
            .destroy_buffer_with_memory(self.staging_buffer, self.staging_buffer_memory);

        self.device.destroy_sampler(self.texture_sampler);
        self.device.destroy_image_view(self.texture_image_view);
        self.device
            .destroy_image_with_memory(self.texture_image, self.texture_image_memory);

        self.device.destroy_descriptor_pool(self.descriptor_pool);

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            self.device.destroy_buffer_with_memory(
                self.uniform_buffers[i],
                self.uniform_buffers_memory[i],
            );
        }

        self.device
            .destroy_buffer_with_memory(self.combined_buffer.buffer, self.combined_buffer.memory);
        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout);
    }
}
