use std::ffi::CStr;

use ash::vk;
use thiserror::Error;

use crate::instance::{Instance, InstanceError};

fn has_required_features(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    let features = instance.get_physical_device_features(physical_device);
    if features.shader_clip_distance != vk::TRUE {
        return false;
    }
    true
}

fn has_required_extensions(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    let mut required_extensions = vec![ash::khr::swapchain::NAME];
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        required_extensions.push(ash::khr::portability_subset::NAME);
    }

    let extensions = instance.get_physical_device_extension_properties(physical_device);
    for required_ext_name_cstr in required_extensions.iter() {
        let required_ext_name = unsafe { CStr::from_ptr(required_ext_name_cstr.as_ptr()) };
        let found = extensions.iter().any(|ext| {
            let available_ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
            available_ext_name == required_ext_name
        });
        if !found {
            return false;
        }
    }
    true
}

#[derive(Debug, Error)]
pub enum PhysicalDeviceManagerError {
    #[error("Failed to enumerate physical devices: {0}")]
    PhysicalDevicesEnumerationError(#[from] InstanceError),

    #[error("No suitable physical device found")]
    NoSuitablePhysicalDevice,
}

pub struct PhysicalDevice {
    pub handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance) -> Result<Self, PhysicalDeviceManagerError> {
        let physical_devices = instance.enumerate_physical_devices()?;
        for d in physical_devices {
            if !has_required_features(instance, d) || !has_required_extensions(instance, d) {
                continue;
            }

            //TODO
        }

        Err(PhysicalDeviceManagerError::NoSuitablePhysicalDevice)
    }
}
