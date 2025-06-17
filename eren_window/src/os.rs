#[derive(Debug)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Web,
}

#[cfg(target_os = "windows")]
pub static CURRENT_PLATFORM: Platform = Platform::Windows;

#[cfg(target_os = "macos")]
pub static CURRENT_PLATFORM: Platform = Platform::MacOS;

#[cfg(target_os = "linux")]
pub static CURRENT_PLATFORM: Platform = Platform::Linux;

#[cfg(target_os = "android")]
pub static CURRENT_PLATFORM: Platform = Platform::Android;

#[cfg(target_os = "ios")]
pub static CURRENT_PLATFORM: Platform = Platform::IOS;

#[cfg(target_arch = "wasm32")]
pub static CURRENT_PLATFORM: Platform = Platform::Web;
