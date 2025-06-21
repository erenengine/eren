use ash::{ext::debug_utils, vk};
use thiserror::Error;

use crate::instance::Instance;

pub fn get_debug_messenger_push_next<'a>() -> vk::DebugUtilsMessengerCreateInfoEXT<'a> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_messenger_callback))
}

unsafe extern "system" fn vulkan_debug_messenger_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = if !(unsafe { *p_callback_data }).p_message.is_null() {
        unsafe { std::ffi::CStr::from_ptr((*p_callback_data).p_message).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed("<<null message>>")
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            log::debug!("[{:#?}] {}", message_type, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("[{:#?}] {}", message_type, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("[{:#?}] {}", message_type, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("[{:#?}] {}", message_type, message);
        }
        _ => {
            log::debug!("[{:#?}] {}", message_type, message);
        }
    }

    vk::FALSE
}

pub struct DebugMessenger {
    debug_utils: debug_utils::Instance,
    handle: vk::DebugUtilsMessengerEXT,
}

#[derive(Debug, Error)]
#[error("Failed to create debug utils messenger: {0}")]
pub struct DebugMessengerCreationError(pub String);

impl DebugMessenger {
    pub fn new(
        instance: &Instance,
        push_next: vk::DebugUtilsMessengerCreateInfoEXT,
    ) -> Result<Self, DebugMessengerCreationError> {
        let debug_utils = instance.create_debug_utils();
        let handle = unsafe {
            debug_utils
                .create_debug_utils_messenger(&push_next, None)
                .map_err(|e| DebugMessengerCreationError(e.to_string()))?
        };
        Ok(Self {
            debug_utils,
            handle,
        })
    }
}

impl Drop for DebugMessenger {
    fn drop(&mut self) {
        log::debug!("Dropping debug messenger");
        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.handle, None);
        }
    }
}
