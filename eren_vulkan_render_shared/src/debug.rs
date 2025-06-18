use ash::{ext::debug_utils, vk};
use thiserror::Error;

use crate::instance::Instance;

unsafe extern "system" fn vulkan_debug_messenger_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*p_callback_data).p_message) };
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}

pub fn get_debug_messenger_push_next<'a>() -> vk::DebugUtilsMessengerCreateInfoEXT<'a> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
            //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_messenger_callback))
}

#[derive(Debug, Error)]
pub enum DebugMessengerError {
    #[error("Failed to create debug utils messenger: {0}")]
    DebugUtilsMessengerCreationError(String),
}

pub struct DebugMessenger {
    debug_utils: debug_utils::Instance,
    handle: vk::DebugUtilsMessengerEXT,
}

impl DebugMessenger {
    pub fn new(
        instance: &Instance,
        push_next: vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> Result<Self, DebugMessengerError> {
        let debug_utils = debug_utils::Instance::new(&instance.entry, &instance.handle);
        let handle = unsafe {
            debug_utils
                .create_debug_utils_messenger(&push_next, None)
                .map_err(|e| DebugMessengerError::DebugUtilsMessengerCreationError(e.to_string()))?
        };
        Ok(Self {
            debug_utils,
            handle,
        })
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        println!("Dropping debug messenger");

        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.handle, None);
        }
    }
}
