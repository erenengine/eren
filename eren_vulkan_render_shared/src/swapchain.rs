use std::sync::Arc;

use crate::{
    device::{Device, FramebufferCreationError, ImageViewCreationError},
    physical_device::PhysicalDevice,
    surface::Surface,
};
use ash::{khr::swapchain, vk};
use thiserror::Error;

pub struct Swapchain {
    // 생명주기 상 스왑체인이 서피스보다 먼저 해제되어야 함
    _surface: Arc<Surface>,

    device: Arc<Device>,
    window_width: u32,
    window_height: u32,
    loader: swapchain::Device,
    handle: vk::SwapchainKHR,
    image_views: Vec<vk::ImageView>,

    pub image_len: usize,
}

#[derive(Debug, Error)]
pub enum SwapchainInitializationError {
    #[error("Failed to create swapchain: {0}")]
    CreateSwapchain(String),

    #[error("Failed to get swapchain images: {0}")]
    GetSwapchainImages(String),

    #[error("Failed to create image view: {0}")]
    CreateImageView(#[from] ImageViewCreationError),
}

#[derive(Debug, Error)]
#[error("Failed to acquire next image: {0}")]
pub struct SwapchainAcquireError(String);

#[derive(Debug, Error)]
#[error("Failed to present swapchain: {0}")]
pub struct SwapchainPresentError(String);

impl Swapchain {
    pub fn new(
        surface: Arc<Surface>,
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        window_width: u32,
        window_height: u32,
        old_swapchain: Option<&Swapchain>,
    ) -> Result<Self, SwapchainInitializationError> {
        let present_queue_family_indices = [
            physical_device
                .queue_family_indices
                .graphics_queue_family_index
                .expect("Graphics queue family index not found"),
            physical_device
                .queue_family_indices
                .present_queue_family_index
                .expect("Present queue family index not found"),
        ];

        let swapchain_info = surface.get_swapchain_info(
            physical_device,
            &present_queue_family_indices,
            window_width,
            window_height,
            old_swapchain.map(|swapchain| swapchain.handle),
        );

        let loader = device.create_swapchain_loader();

        let handle = unsafe {
            loader
                .create_swapchain(&swapchain_info, None)
                .map_err(|e| SwapchainInitializationError::CreateSwapchain(e.to_string()))?
        };

        let images = unsafe { loader.get_swapchain_images(handle) }
            .map_err(|e| SwapchainInitializationError::GetSwapchainImages(e.to_string()))?;

        let mut image_views = Vec::new();
        for &image in &images {
            image_views.push(device.create_image_view(
                image,
                physical_device.preferred_surface_format.format,
                vk::ImageAspectFlags::COLOR,
            )?);
        }

        Ok(Self {
            _surface: surface,
            device,
            window_width,
            window_height,
            loader,
            handle,
            image_views,
            image_len: images.len(),
        })
    }

    pub fn create_framebuffers(
        &self,
        render_pass: vk::RenderPass,
    ) -> Result<Vec<vk::Framebuffer>, FramebufferCreationError> {
        let mut framebuffers = Vec::with_capacity(self.image_views.len());

        for &image_view in self.image_views.iter() {
            let attachments = [image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(self.window_width)
                .height(self.window_height)
                .layers(1);

            let framebuffer = self.device.create_framebuffer(framebuffer_info)?;
            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    pub fn acquire_next_image(
        &self,
        semaphore: vk::Semaphore,
    ) -> Result<(u32, bool), SwapchainAcquireError> {
        Ok(unsafe {
            self.loader
                .acquire_next_image(self.handle, u64::MAX, semaphore, vk::Fence::null())
                .map_err(|e| SwapchainAcquireError(e.to_string()))?
        })
    }

    pub fn present(
        &self,
        present_queue: vk::Queue,
        image_index: u32,
        wait_semaphore: vk::Semaphore,
    ) -> Result<bool, SwapchainPresentError> {
        Ok(unsafe {
            self.loader
                .queue_present(
                    present_queue,
                    &vk::PresentInfoKHR::default()
                        .wait_semaphores(std::slice::from_ref(&wait_semaphore))
                        .swapchains(std::slice::from_ref(&self.handle))
                        .image_indices(std::slice::from_ref(&image_index)),
                )
                .map_err(|e| SwapchainPresentError(e.to_string()))?
        })
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        log::debug!("Dropping swapchain");

        self.device.wait_idle();

        for &image_view in self.image_views.iter() {
            self.device.destroy_image_view(image_view);
        }

        unsafe {
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
