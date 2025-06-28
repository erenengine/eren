use std::sync::Arc;

use ash::{khr, vk};
use thiserror::Error;

use crate::{
    instance::{Instance, SurfaceCreationError},
    physical_device::PhysicalDevice,
};

#[derive(Debug)]
pub struct SurfaceInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceInfo {
    pub fn can_create_swapchain(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn select_preferred_surface_format(&self) -> vk::SurfaceFormatKHR {
        // 드라이버가 “아무 포맷이나 괜찮다”는 의미로 VK_FORMAT_UNDEFINED 하나만 줄 때
        if self.formats.len() == 1 && self.formats[0].format == vk::Format::UNDEFINED {
            return vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_SRGB,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            };
        }

        // 가능한 최선의 선택
        let preferred = [
            (vk::Format::B8G8R8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
            (
                vk::Format::B8G8R8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
            (vk::Format::R8G8B8A8_SRGB, vk::ColorSpaceKHR::SRGB_NONLINEAR),
            (
                vk::Format::R8G8B8A8_UNORM,
                vk::ColorSpaceKHR::SRGB_NONLINEAR,
            ),
        ];

        for &(fmt, cs) in &preferred {
            if let Some(found) = self
                .formats
                .iter()
                .find(|f| f.format == fmt && f.color_space == cs)
            {
                return *found;
            }
        }

        // 아무거나 (보통 0번은 항상 호환)
        self.formats[0]
    }

    pub fn select_preferred_present_mode(&self) -> vk::PresentModeKHR {
        // Prefer MAILBOX mode for lower latency and less tearing
        if self.present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else {
            // FIFO is guaranteed to be available on all platforms
            vk::PresentModeKHR::FIFO
        }
    }
}

pub struct Surface {
    // 생명주기 상 서피스가 인스턴스보다 먼저 해제되어야 함
    _instance: Arc<Instance>,

    loader: Arc<khr::surface::Instance>,
    handle: vk::SurfaceKHR,
}

#[derive(Debug, Error)]
pub enum SurfaceInfoQueryError {
    #[error("Failed to query surface capabilities: {0}")]
    Capabilities(String),

    #[error("Failed to query surface formats: {0}")]
    Formats(String),

    #[error("Failed to query surface present modes: {0}")]
    PresentModes(String),
}

impl Surface {
    pub fn new(instance: Arc<Instance>) -> Result<Self, SurfaceCreationError> {
        let loader = instance.get_surface_loader();
        let handle = instance.create_surface()?;
        Ok(Self {
            _instance: instance,
            loader,
            handle,
        })
    }

    pub fn can_queue_family_present_to_surface(
        &self,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> bool {
        unsafe {
            self.loader
                .get_physical_device_surface_support(
                    physical_device,
                    queue_family_index,
                    self.handle,
                )
                .unwrap_or(false)
        }
    }

    pub fn query_surface_info(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<SurfaceInfo, SurfaceInfoQueryError> {
        let capabilities = unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical_device, self.handle)
                .map_err(|e| SurfaceInfoQueryError::Capabilities(e.to_string()))?
        };

        let formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(physical_device, self.handle)
                .map_err(|e| SurfaceInfoQueryError::Formats(e.to_string()))?
        };

        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(physical_device, self.handle)
                .map_err(|e| SurfaceInfoQueryError::PresentModes(e.to_string()))?
        };

        Ok(SurfaceInfo {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn get_swapchain_info<'a>(
        &self,
        physical_device: &PhysicalDevice,
        present_queue_family_indices: &'a [u32],
        window_width: u32,
        window_height: u32,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Result<vk::SwapchainCreateInfoKHR<'a>, SurfaceInfoQueryError> {
        let surface_format = physical_device.preferred_surface_format;
        let (swapchain_extent, transform) =
            physical_device.query_extent_and_transform(window_width, window_height)?;
        let present_mode = physical_device.preferred_present_mode;

        log::info!("Surface format: {:#?}", surface_format);
        log::info!("Swapchain extent: {:#?}", swapchain_extent);
        log::info!("Present mode: {:#?}", present_mode);

        let mut swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.handle)
            .min_image_count(physical_device.min_swapchain_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(swapchain_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        if let Some(old_swapchain) = old_swapchain {
            swapchain_info = swapchain_info.old_swapchain(old_swapchain);
        }

        if present_queue_family_indices[0] != present_queue_family_indices[1] {
            swapchain_info = swapchain_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(present_queue_family_indices);
        } else {
            swapchain_info = swapchain_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        Ok(swapchain_info)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        log::debug!("Dropping surface");
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
