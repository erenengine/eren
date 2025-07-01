use std::ffi::c_void;
use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    attachment::Attachment,
    device::{
        AttachmentCreationError, BufferWithMemoryCreationError, DescriptorPoolCreationError,
        DescriptorSetAllocationError, DescriptorSetLayoutCreationError, Device,
        FramebufferCreationError, GraphicsPipelineCreationError, MapMemoryError,
        PipelineLayoutCreationError, RenderPassCreationError, SamplerCreationError,
    },
    frame::MAX_FRAMES_IN_FLIGHT,
    physical_device::PhysicalDevice,
    swapchain::Swapchain,
};
use thiserror::Error;

use crate::test_shadow::{
    mesh::{MeshBuffer, Vertex},
    ubo::{LightUBO, MainUBO},
};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("./shaders/main.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("./shaders/main.frag.spv");

const CLEAR_VALUES: [vk::ClearValue; 2] = [
    vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [0.1921, 0.302, 0.4745, 1.0],
        },
    },
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    },
];

pub struct TestMainPass {
    device: Arc<Device>,
    render_area: vk::Rect2D,

    depth_attachment: Attachment,

    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    descriptor_pool: vk::DescriptorPool,

    // Set 0: UBOs (MainUBO, LightUBO)
    descriptor_set_layout_0: vk::DescriptorSetLayout,
    descriptor_sets_0: Vec<vk::DescriptorSet>, // 프레임마다 하나씩

    // Set 1: Shadow Map Sampler
    descriptor_set_layout_1: vk::DescriptorSetLayout,
    descriptor_sets_1: Vec<vk::DescriptorSet>, // 프레임마다 하나씩

    // UBO 버퍼들 (프레임마다 하나씩)
    main_uniform_buffers: Vec<(vk::Buffer, vk::DeviceMemory, *mut c_void)>,
    light_uniform_buffers: Vec<(vk::Buffer, vk::DeviceMemory, *mut c_void)>,

    shadow_map_sampler: vk::Sampler,
}

#[derive(Debug, Error)]
pub enum TestMainPassInitializationError {
    #[error("Failed to create attachment: {0}")]
    CreateAttachment(#[from] AttachmentCreationError),

    #[error("Failed to create descriptor set layout: {0}")]
    CreateDescriptorSetLayout(#[from] DescriptorSetLayoutCreationError),

    #[error("Failed to create pipeline layout: {0}")]
    CreatePipelineLayout(#[from] PipelineLayoutCreationError),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPass(#[from] RenderPassCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(#[from] GraphicsPipelineCreationError),

    #[error("Failed to create framebuffers: {0}")]
    CreateFramebuffers(#[from] FramebufferCreationError),

    #[error("Failed to create uniform buffer: {0}")]
    CreateUniformBuffer(#[from] BufferWithMemoryCreationError),

    #[error("Failed to map memory: {0}")]
    MapMemory(#[from] MapMemoryError),

    #[error("Failed to create descriptor pool: {0}")]
    CreateDescriptorPool(#[from] DescriptorPoolCreationError),

    #[error("Failed to allocate descriptor sets: {0}")]
    AllocateDescriptorSets(#[from] DescriptorSetAllocationError),

    #[error("Failed to create sampler: {0}")]
    CreateSampler(#[from] SamplerCreationError),
}

impl TestMainPass {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        render_area: vk::Rect2D,
        swapchain: &Swapchain,
        shadow_map_view: vk::ImageView, // ShadowPass에서 생성된 뎁스맵 뷰
    ) -> Result<Self, TestMainPassInitializationError> {
        // 셰이더는 두 개의 디스크립터 셋을 사용합니다 (set=0, set=1)

        // Set 0: MainUBO와 LightUBO를 위한 레이아웃
        let main_ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let light_ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout_0 =
            device.create_descriptor_set_layout(&[main_ubo_binding, light_ubo_binding])?;

        // Set 1: 그림자 맵 샘플러를 위한 레이아웃
        let shadow_sampler_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout_1 =
            device.create_descriptor_set_layout(&[shadow_sampler_binding])?;

        // 컬러 어태치먼트(스왑체인)와 뎁스 어태치먼트(자체 생성)를 사용합니다.
        let color_attachment = device.get_swapchain_color_attachment_desc();

        let depth_attachment = device.create_depth_attachment(
            swapchain.extent,
            physical_device.depth_format,
            physical_device.uses_stencil,
            vk::SampleCountFlags::TYPE_1,
            false, // 이 뎁스 버퍼는 샘플링하지 않음
        )?;

        let color_attachment_ref = device.get_color_attachment_ref(0);
        let depth_attachment_ref = device.get_depth_attachment_ref(1, physical_device.uses_stencil);

        let subpass = vk::SubpassDescription2::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);

        let attachments = [color_attachment, depth_attachment.desc];
        let render_pass = device.create_render_pass(&attachments, &[subpass], &[])?;

        let pipeline_layout = device
            .create_pipeline_layout(&[descriptor_set_layout_0, descriptor_set_layout_1], &[])?;

        let pipeline = Self::create_pipeline(
            device.clone(),
            pipeline_layout,
            render_pass,
            swapchain.extent,
        )?;

        let swapchain_framebuffers = swapchain
            .create_framebuffers_with_depth_image_view(render_pass, depth_attachment.view)?;

        let main_uniform_buffers = Self::create_uniform_buffers(
            device.clone(),
            MAX_FRAMES_IN_FLIGHT,
            std::mem::size_of::<MainUBO>(),
        )?;
        let light_uniform_buffers = Self::create_uniform_buffers(
            device.clone(),
            MAX_FRAMES_IN_FLIGHT,
            std::mem::size_of::<LightUBO>(),
        )?;

        // 그림자 맵 샘플러 생성. Compare Op를 사용하는 것이 PCF(Percentage-Closer Filtering)에 더 좋지만,
        // 맥에서 MoltenVK는 Compare Op를 지원하지 않으므로 일단 이렇게 둡니다.
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_BORDER)
            .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
            .compare_enable(false); // PCF를 위해서는 true로 설정
        let shadow_map_sampler = device.create_sampler(&sampler_info)?;

        // 디스크립터 풀 생성
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: (MAX_FRAMES_IN_FLIGHT * 2) as u32,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: MAX_FRAMES_IN_FLIGHT as u32,
            },
        ];
        let descriptor_pool =
            device.create_descriptor_pool((MAX_FRAMES_IN_FLIGHT * 2) as u32, &pool_sizes)?;

        // 디스크립터 셋 할당 및 쓰기
        let layouts_0 = vec![descriptor_set_layout_0; MAX_FRAMES_IN_FLIGHT];
        let layouts_1 = vec![descriptor_set_layout_1; MAX_FRAMES_IN_FLIGHT];
        let descriptor_sets_0 = device.allocate_descriptor_sets(descriptor_pool, &layouts_0)?;
        let descriptor_sets_1 = device.allocate_descriptor_sets(descriptor_pool, &layouts_1)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            // Set 0 업데이트 (UBOs)
            let main_ubo_info = vk::DescriptorBufferInfo {
                buffer: main_uniform_buffers[i].0,
                offset: 0,
                range: std::mem::size_of::<MainUBO>() as vk::DeviceSize,
            };
            let light_ubo_info = vk::DescriptorBufferInfo {
                buffer: light_uniform_buffers[i].0,
                offset: 0,
                range: std::mem::size_of::<LightUBO>() as vk::DeviceSize,
            };

            let write_main_ubo = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets_0[i])
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&main_ubo_info));

            let write_light_ubo = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets_0[i])
                .dst_binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&light_ubo_info));

            // Set 1 업데이트 (Shadow Map)
            let shadow_image_info = vk::DescriptorImageInfo {
                sampler: shadow_map_sampler,
                image_view: shadow_map_view,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            };

            let write_shadow_map = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets_1[i])
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(std::slice::from_ref(&shadow_image_info));

            device.write_descriptor_sets(&[write_main_ubo, write_light_ubo, write_shadow_map]);
        }

        Ok(Self {
            device,
            render_area,

            depth_attachment,

            render_pass,
            pipeline_layout,
            pipeline,
            swapchain_framebuffers,

            descriptor_pool,
            descriptor_set_layout_0,
            descriptor_sets_0,
            descriptor_set_layout_1,
            descriptor_sets_1,

            main_uniform_buffers,
            light_uniform_buffers,

            shadow_map_sampler,
        })
    }

    // UBO 업데이트를 위한 public 함수들
    pub fn update_main_ubo(&self, frame_index: usize, ubo: MainUBO) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                &ubo,
                self.main_uniform_buffers[frame_index].2 as *mut MainUBO,
                1,
            );
        }
    }

    pub fn update_light_ubo(&self, frame_index: usize, ubo: LightUBO) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                &ubo,
                self.light_uniform_buffers[frame_index].2 as *mut LightUBO,
                1,
            );
        }
    }

    pub fn record_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image_idx: usize,
        frame_index: usize,    // 현재 프레임 인덱스 (UBO/Descriptor 선택용)
        meshes: &[MeshBuffer], // (모델행렬, 메시)
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

        // 두 개의 디스크립터 셋을 바인딩
        self.device.bind_graphics_descriptor_sets(
            command_buffer,
            self.pipeline_layout,
            &[
                self.descriptor_sets_0[frame_index],
                self.descriptor_sets_1[frame_index],
            ],
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

    fn create_pipeline(
        device: Arc<Device>,
        pipeline_layout: vk::PipelineLayout,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<vk::Pipeline, GraphicsPipelineCreationError> {
        let binding_descriptions = [Vertex::get_binding_description()];
        let attribute_descriptions = Vertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        device.create_graphics_pipeline(
            pipeline_info,
            Some(VERT_SHADER_BYTES),
            Some(FRAG_SHADER_BYTES),
        )
    }

    fn create_uniform_buffers(
        device: Arc<Device>,
        num_buffers: usize,
        size: usize,
    ) -> Result<Vec<(vk::Buffer, vk::DeviceMemory, *mut c_void)>, BufferWithMemoryCreationError>
    {
        let mut buffers = Vec::with_capacity(num_buffers);
        for _ in 0..num_buffers {
            let (buffer, memory) = device.create_buffer_with_memory(
                size as vk::DeviceSize,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            let mapped = device.map_memory(memory, size as vk::DeviceSize).unwrap();
            buffers.push((buffer, memory, mapped));
        }
        Ok(buffers)
    }
}

impl Drop for TestMainPass {
    fn drop(&mut self) {
        self.device.wait_idle();

        self.device.destroy_sampler(self.shadow_map_sampler);

        for (buffer, memory, _) in self.main_uniform_buffers.drain(..) {
            self.device.destroy_buffer_with_memory(buffer, memory);
        }
        for (buffer, memory, _) in self.light_uniform_buffers.drain(..) {
            self.device.destroy_buffer_with_memory(buffer, memory);
        }

        self.device.destroy_descriptor_pool(self.descriptor_pool);
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout_0);
        self.device
            .destroy_descriptor_set_layout(self.descriptor_set_layout_1);

        for &framebuffer in self.swapchain_framebuffers.iter() {
            self.device.destroy_framebuffer(framebuffer);
        }

        self.device.destroy_pipeline(self.pipeline);
        self.device.destroy_pipeline_layout(self.pipeline_layout);
        self.device.destroy_render_pass(self.render_pass);
        self.device.destroy_attachment(&self.depth_attachment);
    }
}
