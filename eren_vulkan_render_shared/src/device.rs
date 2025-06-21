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
    // 소멸 순서를 맞추기 위해 보유 (논리 디바이스가 물리 디바이스보다 먼저 소멸해야 함)
    _physical_device: Arc<PhysicalDevice>,

    handle: ash::Device,
    graphics_queue: Option<vk::Queue>,
    compute_queue: Option<vk::Queue>,
    transfer_queue: Option<vk::Queue>,
    sparse_binding_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
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
            _physical_device: physical_device,
            handle,
            graphics_queue,
            compute_queue,
            transfer_queue,
            sparse_binding_queue,
            present_queue,
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
