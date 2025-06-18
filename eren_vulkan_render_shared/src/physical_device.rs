use ash::vk;
use thiserror::Error;

use crate::instance::{Instance, InstanceError};

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
        for physical_device in physical_devices {
            //TODO
        }

        Err(PhysicalDeviceManagerError::NoSuitablePhysicalDevice)
    }
}
