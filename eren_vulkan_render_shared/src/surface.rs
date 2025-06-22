use std::sync::Arc;

use ash::{khr, vk};
use thiserror::Error;

use crate::instance::{Instance, SurfaceCreationError};

#[derive(Debug)]
pub struct SurfaceInfo {
    capabilities: vk::SurfaceCapabilitiesKHR,
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
    pub fn new(instance: &Instance) -> Result<Self, SurfaceCreationError> {
        let loader = instance.get_surface_loader();
        let handle = instance.create_surface()?;
        Ok(Self { loader, handle })
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
}

impl Drop for Surface {
    fn drop(&mut self) {
        log::debug!("Dropping surface");
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
