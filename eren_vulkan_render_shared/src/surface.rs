use std::sync::Arc;

use ash::{khr, vk};
use thiserror::Error;

use crate::instance::{Instance, SurfaceCreationError};

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

#[derive(Debug)]
pub struct SurfaceInfo {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SurfaceInfo {
    pub fn can_create_swapchain(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

impl Surface {
    pub fn new(raw_instance: &mut Instance) -> Result<Self, SurfaceCreationError> {
        let loader = raw_instance.get_surface_loader();
        let handle = raw_instance.create_surface()?;

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
