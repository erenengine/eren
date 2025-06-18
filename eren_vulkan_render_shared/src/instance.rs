use std::{ffi::CString, sync::Arc};

use ash::{ext::debug_utils, vk};
use thiserror::Error;
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

use crate::debug::{DebugMessenger, DebugMessengerError, get_debug_messenger_push_next};

pub struct Instance {
    window: Arc<Window>,
    entry: ash::Entry,
    handle: ash::Instance,

    debug_messenger: Option<DebugMessenger>,
}

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Failed to load entry: {0}")]
    EntryLoadError(#[from] ash::LoadingError),

    #[error("Failed to enumerate required extensions: {0}")]
    ExtensionEnumerationError(String),

    #[error("Failed to create handle: {0}")]
    HandleCreationError(String),

    #[error("Failed to create debug messenger: {0}")]
    DebugMessengerCreationError(#[from] DebugMessengerError),

    #[error("Failed to enumerate physical devices: {0}")]
    EnumeratePhysicalDevicesError(String),
}

impl Instance {
    pub fn new(window: Arc<Window>) -> Result<Self, InstanceError> {
        let entry = unsafe { ash::Entry::load()? };
        let debug_messenger_push_next = get_debug_messenger_push_next();
        let handle = Self::create_handle(window.clone(), &entry, debug_messenger_push_next)?;

        let mut instance = Self {
            window,
            entry,
            handle,
            debug_messenger: None,
        };

        instance.debug_messenger = Some(DebugMessenger::new(&instance, debug_messenger_push_next)?);

        Ok(instance)
    }

    fn create_handle(
        window: Arc<Window>,
        entry: &ash::Entry,
        mut debug_messenger_push_next: vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> Result<ash::Instance, InstanceError> {
        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
        let enabled_layers = vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()];

        let mut enabled_extensions =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .map_err(|e| InstanceError::ExtensionEnumerationError(e.to_string()))?
                .to_vec();
        enabled_extensions.push(debug_utils::NAME.as_ptr());

        let mut flags = vk::InstanceCreateFlags::empty();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            use ash::khr::{get_physical_device_properties2, portability_enumeration};

            enabled_extensions.push(portability_enumeration::NAME.as_ptr());
            enabled_extensions.push(get_physical_device_properties2::NAME.as_ptr());

            flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        let enabled_layers_pointers = enabled_layers
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        let handle_info = vk::InstanceCreateInfo::default()
            .push_next(&mut debug_messenger_push_next)
            .application_info(&app_info)
            .enabled_layer_names(&enabled_layers_pointers)
            .enabled_extension_names(&enabled_extensions)
            .flags(flags);

        unsafe {
            entry
                .create_instance(&handle_info, None)
                .map_err(|e| InstanceError::HandleCreationError(e.to_string()))
        }
    }

    pub fn create_debug_utils(&self) -> debug_utils::Instance {
        debug_utils::Instance::new(&self.entry, &self.handle)
    }

    pub fn enumerate_physical_devices(&self) -> Result<Vec<vk::PhysicalDevice>, InstanceError> {
        unsafe {
            self.handle
                .enumerate_physical_devices()
                .map_err(|e| InstanceError::EnumeratePhysicalDevicesError(e.to_string()))
        }
    }

    pub fn get_physical_device_features(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceFeatures {
        unsafe { self.handle.get_physical_device_features(physical_device) }
    }

    pub fn get_physical_device_extension_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Vec<vk::ExtensionProperties> {
        unsafe {
            self.handle
                .enumerate_device_extension_properties(physical_device)
                .unwrap_or_else(|_| Vec::new())
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            // 인스턴스 소멸 전에 debug messenger 소멸
            self.debug_messenger = None;

            self.handle.destroy_instance(None);
        }
    }
}
