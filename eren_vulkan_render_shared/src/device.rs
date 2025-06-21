use std::sync::Arc;

use ash::vk;

use crate::{
    instance::DeviceCreationError,
    physical_device::{
        PhysicalDevice, get_required_physical_device_extensions,
        get_required_physical_device_features,
    },
};

pub struct Device {
    // 소멸 순서를 맞추기 위해 Arc로 유지 (논리 디바이스가 물리 디바이스보다 먼저 소멸)
    _physical_device: Arc<PhysicalDevice>,
    handle: ash::Device,
}

impl Device {
    pub fn new(physical_device: Arc<PhysicalDevice>) -> Result<Self, DeviceCreationError> {
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

        Ok(Self {
            _physical_device: physical_device,
            handle,
        })
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
