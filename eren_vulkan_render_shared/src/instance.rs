use std::{ffi::CString, sync::Arc};

use ash::{ext::debug_utils, khr, vk};
use thiserror::Error;
use winit::{
    raw_window_handle::{HandleError, HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::debug::{DebugMessenger, DebugMessengerCreationError, get_debug_messenger_push_next};

pub struct Instance {
    window: Arc<Window>,
    entry: ash::Entry,
    handle: ash::Instance,

    debug_messenger: Option<DebugMessenger>,
}

#[derive(Debug, Error)]
pub enum InstanceInitializationError {
    #[error("Failed to load Vulkan entry: {0}")]
    LoadEntry(#[from] ash::LoadingError),

    #[error("Failed to get required extensions: {0}")]
    EnumerateExtensions(String),

    #[error("Failed to create Vulkan instance: {0}")]
    CreateHandle(String),

    #[error("Failed to create debug messenger: {0}")]
    CreateDebugMessenger(#[from] DebugMessengerCreationError),
}

#[derive(Debug, Error)]
pub enum SurfaceCreationError {
    #[error("Failed to get window handle: {0}")]
    GetWindowHandle(#[from] HandleError),

    #[error("Failed to create surface: {0}")]
    CreateSurface(String),
}

#[derive(Debug, Error)]
#[error("Failed to enumerate physical devices: {0}")]
pub struct PhysicalDevicesEnumerationError(pub String);

#[derive(Debug, Error)]
#[error("Failed to create device: {0}")]
pub struct DeviceCreationError(pub String);

impl Instance {
    pub fn new(window: Arc<Window>) -> Result<Self, InstanceInitializationError> {
        let entry = unsafe { ash::Entry::load()? };

        #[cfg(debug_assertions)]
        let debug_messenger_push_next = get_debug_messenger_push_next();

        let handle = Self::create_handle(
            window.clone(),
            &entry,
            #[cfg(debug_assertions)]
            Some(debug_messenger_push_next),
            #[cfg(not(debug_assertions))]
            None,
        )?;

        let mut instance = Self {
            window,
            entry,
            handle,
            debug_messenger: None,
        };

        #[cfg(debug_assertions)]
        {
            instance.debug_messenger =
                Some(DebugMessenger::new(&instance, debug_messenger_push_next)?);
        }

        Ok(instance)
    }

    fn create_handle(
        window: Arc<Window>,
        entry: &ash::Entry,
        #[cfg(debug_assertions)] mut debug_messenger_push_next: Option<
            vk::DebugUtilsMessengerCreateInfoEXT,
        >,
        #[cfg(not(debug_assertions))] _debug_messenger_push_next: Option<()>,
    ) -> Result<ash::Instance, InstanceInitializationError> {
        let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        #[cfg(debug_assertions)]
        let enabled_layers = vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        #[cfg(not(debug_assertions))]
        let enabled_layers = vec![];

        let mut enabled_extensions =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .map_err(|e| InstanceInitializationError::EnumerateExtensions(e.to_string()))?
                .to_vec();

        #[cfg(debug_assertions)]
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

        let mut handle_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&enabled_layers_pointers)
            .enabled_extension_names(&enabled_extensions)
            .flags(flags);

        #[cfg(debug_assertions)]
        if let Some(ref mut push_next) = debug_messenger_push_next {
            handle_info = handle_info.push_next(push_next);
        }

        unsafe {
            entry
                .create_instance(&handle_info, None)
                .map_err(|e| InstanceInitializationError::CreateHandle(e.to_string()))
        }
    }

    pub fn create_debug_utils(&self) -> debug_utils::Instance {
        debug_utils::Instance::new(&self.entry, &self.handle)
    }

    pub fn create_surface_loader(&self) -> khr::surface::Instance {
        khr::surface::Instance::new(&self.entry, &self.handle)
    }

    pub fn create_surface(&self) -> Result<vk::SurfaceKHR, SurfaceCreationError> {
        let display_handle = self.window.display_handle()?;
        let window_handle = self.window.window_handle()?;

        let surface = unsafe {
            ash_window::create_surface(
                &self.entry,
                &self.handle,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
            .map_err(|e| SurfaceCreationError::CreateSurface(e.to_string()))?
        };

        Ok(surface)
    }

    pub fn get_physical_devices(
        &self,
    ) -> Result<Vec<vk::PhysicalDevice>, PhysicalDevicesEnumerationError> {
        unsafe {
            self.handle
                .enumerate_physical_devices()
                .map_err(|e| PhysicalDevicesEnumerationError(e.to_string()))
        }
    }

    pub fn get_physical_device_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceProperties {
        unsafe { self.handle.get_physical_device_properties(physical_device) }
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

    pub fn get_physical_device_queue_family_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.handle
                .get_physical_device_queue_family_properties(physical_device)
        }
    }

    pub fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        info: vk::DeviceCreateInfo,
    ) -> Result<ash::Device, DeviceCreationError> {
        unsafe {
            self.handle
                .create_device(physical_device, &info, None)
                .map_err(|e| DeviceCreationError(e.to_string()))
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        // 인스턴스 소멸 전에 debug messenger 소멸
        self.debug_messenger = None;

        log::debug!("Dropping instance");

        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}
