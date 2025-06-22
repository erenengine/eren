use std::{io::Cursor, sync::Arc};

use ash::{khr::swapchain, util, vk};
use thiserror::Error;

use crate::{
    instance::{DeviceCreationError, Instance},
    physical_device::{
        MemoryTypeIndexNotFoundError, PhysicalDevice, get_required_physical_device_extensions,
        get_required_physical_device_features,
    },
    swapchain::{Swapchain, SwapchainPresentError},
};

pub struct Device {
    instance: Arc<Instance>,
    physical_device: Arc<PhysicalDevice>,

    handle: ash::Device,
    graphics_queue: Option<vk::Queue>,
    compute_queue: Option<vk::Queue>,
    transfer_queue: Option<vk::Queue>,
    sparse_binding_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
}

#[derive(Debug, Error)]
#[error("Failed to create command pool: {0}")]
pub struct CommandPoolCreationError(String);

#[derive(Debug, Error)]
#[error("Failed to allocate command buffer: {0}")]
pub struct CommandBufferAllocationError(String);

#[derive(Debug, Error)]
#[error("Failed to reset command buffer: {0}")]
pub struct CommandBufferResetError(String);

#[derive(Debug, Error)]
#[error("Failed to create semaphore: {0}")]
pub struct SemaphoreCreationError(String);

#[derive(Debug, Error)]
#[error("Failed to create fence: {0}")]
pub struct FenceCreationError(String);

#[derive(Debug, Error)]
#[error("Failed to wait for fences: {0}")]
pub struct WaitForFencesError(String);

#[derive(Debug, Error)]
pub enum ImageWithMemoryCreationError {
    #[error("Failed to create image: {0}")]
    CreateImage(String),

    #[error("Failed to find memory type index: {0}")]
    FindMemoryTypeIndex(#[from] MemoryTypeIndexNotFoundError),

    #[error("Failed to allocate memory: {0}")]
    AllocateMemory(String),

    #[error("Failed to bind memory to image: {0}")]
    BindMemoryToImage(String),
}

#[derive(Debug, Error)]
#[error("Failed to create image view: {0}")]
pub struct ImageViewCreationError(String);

pub struct Attachment<'a> {
    pub desc: vk::AttachmentDescription2<'a>,
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

#[derive(Debug, Error)]
pub enum AttachmentCreationError {
    #[error("Failed to create image with memory: {0}")]
    CreateImageWithMemory(#[from] ImageWithMemoryCreationError),

    #[error("Failed to create image view: {0}")]
    CreateImageView(#[from] ImageViewCreationError),
}

#[derive(Debug, Error)]
#[error("Failed to create render pass: {0}")]
pub struct RenderPassCreationError(String);

#[derive(Debug, Error)]
#[error("Failed to create framebuffer: {0}")]
pub struct FramebufferCreationError(String);

#[derive(Debug, Error)]
pub enum ShaderModuleCreationError {
    #[error("Failed to read SPIR-V bytecode: {0}")]
    ReadSpv(String),

    #[error("Failed to create shader module: {0}")]
    CreateShaderModule(String),
}

#[derive(Debug, Error)]
pub enum GraphicsPipelineCreationError {
    #[error("Failed to create shader module: {0}")]
    CreateShaderModule(#[from] ShaderModuleCreationError),

    #[error("Failed to create graphics pipeline: {0}")]
    CreateGraphicsPipeline(String),
}

#[derive(Debug, Error)]
#[error("Failed to submit graphics commands: {0}")]
pub struct SubmitGraphicsCommandsError(String);

impl Device {
    pub fn new(
        instance: Arc<Instance>,
        physical_device: Arc<PhysicalDevice>,
    ) -> Result<Self, DeviceCreationError> {
        let queue_infos = physical_device.get_queue_infos();
        let required_features = get_required_physical_device_features();
        let required_extensions = get_required_physical_device_extensions();
        let required_extensions_pointers = required_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_infos)
            .enabled_features(&required_features)
            .enabled_extension_names(&required_extensions_pointers);

        let handle = physical_device.create_device(device_info)?;

        let graphics_queue = physical_device
            .queue_family_indices
            .graphics_queue_family_index
            .map(|index| unsafe { handle.get_device_queue(index, 0) });

        let compute_queue = physical_device
            .queue_family_indices
            .compute_queue_family_index
            .map(|index| unsafe { handle.get_device_queue(index, 0) });

        let transfer_queue = physical_device
            .queue_family_indices
            .transfer_queue_family_index
            .map(|index| unsafe { handle.get_device_queue(index, 0) });

        let sparse_binding_queue = physical_device
            .queue_family_indices
            .sparse_binding_queue_family_index
            .map(|index| unsafe { handle.get_device_queue(index, 0) });

        let present_queue = physical_device
            .queue_family_indices
            .present_queue_family_index
            .map(|index| unsafe { handle.get_device_queue(index, 0) });

        Ok(Self {
            instance,
            physical_device,
            handle,
            graphics_queue,
            compute_queue,
            transfer_queue,
            sparse_binding_queue,
            present_queue,
        })
    }

    pub fn wait_idle(&self) {
        unsafe {
            self.handle
                .device_wait_idle()
                .expect("Failed to wait for device idle");
        }
    }

    pub fn create_swapchain_loader(&self) -> swapchain::Device {
        self.instance.create_swapchain_loader(&self.handle)
    }

    pub fn create_command_pool(&self) -> Result<vk::CommandPool, CommandPoolCreationError> {
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(
                self.physical_device
                    .queue_family_indices
                    .graphics_queue_family_index
                    .expect("Graphics queue family index not found"),
            )
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        Ok(unsafe {
            self.handle
                .create_command_pool(&command_pool_info, None)
                .map_err(|e| CommandPoolCreationError(e.to_string()))?
        })
    }

    pub fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        unsafe {
            self.handle.destroy_command_pool(command_pool, None);
        }
    }

    pub fn allocate_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
    ) -> Result<Vec<vk::CommandBuffer>, CommandBufferAllocationError> {
        Ok(unsafe {
            self.handle
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .command_buffer_count(command_buffer_count),
                )
                .map_err(|e| CommandBufferAllocationError(e.to_string()))?
        })
    }

    pub fn reset_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
    ) -> Result<(), CommandBufferResetError> {
        unsafe {
            self.handle
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .map_err(|e| CommandBufferResetError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn free_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffers: Vec<vk::CommandBuffer>,
    ) {
        unsafe {
            self.handle
                .free_command_buffers(command_pool, &command_buffers);
        }
    }

    pub fn create_semaphore(&self) -> Result<vk::Semaphore, SemaphoreCreationError> {
        Ok(unsafe {
            self.handle
                .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                .map_err(|e| SemaphoreCreationError(e.to_string()))?
        })
    }

    pub fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe {
            self.handle.destroy_semaphore(semaphore, None);
        }
    }

    pub fn create_fence_signaled(&self) -> Result<vk::Fence, FenceCreationError> {
        Ok(unsafe {
            self.handle
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .map_err(|e| FenceCreationError(e.to_string()))?
        })
    }

    pub fn wait_for_fence(&self, fence: vk::Fence) -> Result<(), WaitForFencesError> {
        unsafe {
            self.handle
                .wait_for_fences(&[fence], true, u64::MAX)
                .map_err(|e| WaitForFencesError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn destroy_fence(&self, fence: vk::Fence) {
        unsafe {
            self.handle.destroy_fence(fence, None);
        }
    }

    pub fn create_image_with_memory(
        &self,
        format: vk::Format,
        extent: vk::Extent2D,
        samples: vk::SampleCountFlags,
        usage: vk::ImageUsageFlags,
        tiling: vk::ImageTiling,
        memory_properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Image, vk::DeviceMemory), ImageWithMemoryCreationError> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(samples)
            .tiling(tiling)
            .usage(usage)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe {
            self.handle
                .create_image(&image_info, None)
                .map_err(|e| ImageWithMemoryCreationError::CreateImage(e.to_string()))?
        };

        let memory_requirements = unsafe { self.handle.get_image_memory_requirements(image) };

        let memory_type_index = self
            .physical_device
            .find_memory_type_index(memory_requirements.memory_type_bits, memory_properties)?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe { self.handle.allocate_memory(&alloc_info, None) }
            .map_err(|e| ImageWithMemoryCreationError::AllocateMemory(e.to_string()))?;

        unsafe {
            self.handle
                .bind_image_memory(image, memory, 0)
                .map_err(|e| ImageWithMemoryCreationError::BindMemoryToImage(e.to_string()))?;
        }

        Ok((image, memory))
    }

    pub fn create_image_view(
        &self,
        image: vk::Image,
        format: vk::Format,
        aspect: vk::ImageAspectFlags,
    ) -> Result<vk::ImageView, ImageViewCreationError> {
        let info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: aspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        Ok(unsafe {
            self.handle
                .create_image_view(&info, None)
                .map_err(|e| ImageViewCreationError(e.to_string()))?
        })
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.handle.destroy_image_view(image_view, None);
        }
    }

    pub fn create_depth_attachment(
        &self,
        extent: vk::Extent2D,
        format: vk::Format,
        samples: vk::SampleCountFlags,
        sampled: bool,
    ) -> Result<Attachment, AttachmentCreationError> {
        let usage = vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
            | if sampled {
                vk::ImageUsageFlags::SAMPLED
            } else {
                vk::ImageUsageFlags::empty()
            };

        let (image, memory) = self.create_image_with_memory(
            format,
            extent,
            samples,
            usage,
            vk::ImageTiling::OPTIMAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let view = self.create_image_view(
            image,
            format,
            vk::ImageAspectFlags::DEPTH, // 스텐실을 사용하지 않음
        )?;

        let desc = vk::AttachmentDescription2::default()
            .format(format)
            .samples(samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(if sampled {
                vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL
            } else {
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            });

        Ok(Attachment {
            desc,
            image,
            memory,
            view,
        })
    }

    pub fn create_color_attachment(
        &self,
        extent: vk::Extent2D,
        format: vk::Format,
        samples: vk::SampleCountFlags,
        sampled: bool,
        clear_on_load: bool,
    ) -> Result<Attachment, AttachmentCreationError> {
        let usage = vk::ImageUsageFlags::COLOR_ATTACHMENT
            | if sampled {
                vk::ImageUsageFlags::SAMPLED
            } else {
                vk::ImageUsageFlags::empty()
            };

        let (image, memory) = self.create_image_with_memory(
            format,
            extent,
            samples,
            usage,
            vk::ImageTiling::OPTIMAL,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let view = self.create_image_view(image, format, vk::ImageAspectFlags::COLOR)?;

        let desc = vk::AttachmentDescription2::default()
            .format(format)
            .samples(samples)
            .load_op(if clear_on_load {
                vk::AttachmentLoadOp::CLEAR
            } else {
                vk::AttachmentLoadOp::LOAD
            })
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(if sampled {
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            } else {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            });

        Ok(Attachment {
            desc,
            image,
            memory,
            view,
        })
    }

    pub fn get_swapchain_color_attachment_desc(&self) -> vk::AttachmentDescription2 {
        vk::AttachmentDescription2::default()
            .format(self.physical_device.preferred_surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1) // swapchain 은 1x MSAA
            .load_op(vk::AttachmentLoadOp::CLEAR) // 프레임 시작 시 항상 Clear
            .store_op(vk::AttachmentStoreOp::STORE) // 화면에 보여야 하므로 저장
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED) // 첫 사용 이전 상태 미정
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR) // 최종 레이아웃은 Present
    }

    pub fn get_depth_attachment_ref(&self, attachment_index: u32) -> vk::AttachmentReference2 {
        vk::AttachmentReference2::default()
            .attachment(attachment_index)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::DEPTH) // 스텐실을 사용하지 않음
    }

    pub fn get_color_attachment_ref(&self, attachment_index: u32) -> vk::AttachmentReference2 {
        vk::AttachmentReference2::default()
            .attachment(attachment_index)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR)
    }

    pub fn destroy_attachment(&self, attachment: Attachment) {
        unsafe {
            self.handle.destroy_image_view(attachment.view, None);
            self.handle.destroy_image(attachment.image, None);
            self.handle.free_memory(attachment.memory, None);
        }
    }

    pub fn create_render_pass(
        &self,
        attachments: &[vk::AttachmentDescription2],
        subpasses: &[vk::SubpassDescription2],
        subpass_dependencies: &[vk::SubpassDependency2],
    ) -> Result<vk::RenderPass, RenderPassCreationError> {
        let create_info = vk::RenderPassCreateInfo2::default()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(subpass_dependencies);

        unsafe {
            self.handle
                .create_render_pass2(&create_info, None)
                .map_err(|e| RenderPassCreationError(e.to_string()))
        }
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe {
            self.handle.destroy_render_pass(render_pass, None);
        }
    }

    pub fn create_framebuffer(
        &self,
        framebuffer_info: vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, FramebufferCreationError> {
        Ok(unsafe {
            self.handle
                .create_framebuffer(&framebuffer_info, None)
                .map_err(|e| FramebufferCreationError(e.to_string()))?
        })
    }

    pub fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        unsafe {
            self.handle.destroy_framebuffer(framebuffer, None);
        }
    }

    fn create_shader_module(
        &self,
        code: &[u8],
    ) -> Result<vk::ShaderModule, ShaderModuleCreationError> {
        let mut cursor = Cursor::new(code);
        let code = util::read_spv(&mut cursor)
            .map_err(|e| ShaderModuleCreationError::ReadSpv(e.to_string()))?;

        let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

        Ok(unsafe {
            self.handle
                .create_shader_module(&create_info, None)
                .map_err(|e| ShaderModuleCreationError::CreateShaderModule(e.to_string()))?
        })
    }

    fn destroy_shader_module(&self, module: vk::ShaderModule) {
        unsafe {
            self.handle.destroy_shader_module(module, None);
        }
    }

    pub fn create_graphics_pipeline(
        &self,
        create_info: vk::GraphicsPipelineCreateInfo,
        vert_shader_bytes: Option<&[u8]>,
        frag_shader_bytes: Option<&[u8]>,
    ) -> Result<vk::Pipeline, GraphicsPipelineCreationError> {
        let vert_shader_module = match vert_shader_bytes {
            Some(bytes) => Some(self.create_shader_module(bytes)?),
            None => None,
        };

        let frag_shader_module = match frag_shader_bytes {
            Some(bytes) => Some(self.create_shader_module(bytes)?),
            None => None,
        };

        let mut shader_stages = Vec::new();
        let main_function_name = std::ffi::CString::new("main").unwrap();

        if let Some(module) = vert_shader_module {
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::VERTEX)
                    .module(module)
                    .name(&main_function_name),
            );
        }

        if let Some(module) = frag_shader_module {
            shader_stages.push(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::FRAGMENT)
                    .module(module)
                    .name(&main_function_name),
            );
        }

        let pipeline = unsafe {
            self.handle
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[create_info.stages(&shader_stages)],
                    None,
                )
                .map_err(|e| {
                    GraphicsPipelineCreationError::CreateGraphicsPipeline(e.1.to_string())
                })?
        }[0];

        if let Some(module) = vert_shader_module {
            self.destroy_shader_module(module);
        }

        if let Some(module) = frag_shader_module {
            self.destroy_shader_module(module);
        }

        Ok(pipeline)
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe {
            self.handle.destroy_pipeline(pipeline, None);
        }
    }

    pub fn submit_graphics_commands(
        &self,
        command_buffer: vk::CommandBuffer,
        image_available_semaphore: vk::Semaphore,
        render_finished_semaphore: vk::Semaphore,
        in_flight_fence: vk::Fence,
    ) -> Result<(), SubmitGraphicsCommandsError> {
        // 지난 프레임에서 signal 된 펜스를 다시 쓸 수 있게 reset
        unsafe {
            self.handle
                .reset_fences(&[in_flight_fence])
                .map_err(|e| SubmitGraphicsCommandsError(e.to_string()))?;
        }

        // 세마포어-대상 스테이지 매핑
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(std::slice::from_ref(&image_available_semaphore))
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&command_buffer))
            .signal_semaphores(std::slice::from_ref(&render_finished_semaphore));

        // 제출
        unsafe {
            self.handle.queue_submit(
                self.graphics_queue.expect("Graphics queue not found"),
                &[submit_info],
                in_flight_fence,
            )
        }
        .map_err(|e| SubmitGraphicsCommandsError(e.to_string()))
    }

    pub fn present(
        &self,
        swapchain: &Swapchain,
        image_index: u32,
        semaphore: vk::Semaphore,
    ) -> Result<bool, SwapchainPresentError> {
        swapchain.present(
            self.present_queue.expect("Present queue not found"),
            image_index,
            semaphore,
        )
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        log::debug!("Dropping device");
        unsafe {
            self.handle.destroy_device(None);
        }
    }
}
