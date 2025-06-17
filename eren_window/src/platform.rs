#[derive(Debug, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Android,
    IOS,
    Web,
    Unknown,
}

pub static CURRENT_PLATFORM: Platform = {
    #[cfg(target_arch = "wasm32")]
    {
        Platform::Web
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "windows"))]
    {
        Platform::Windows
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
    {
        Platform::MacOS
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "linux"))]
    {
        Platform::Linux
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "android"))]
    {
        Platform::Android
    }

    #[cfg(all(not(target_arch = "wasm32"), target_os = "ios"))]
    {
        Platform::IOS
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "windows"),
        not(target_os = "macos"),
        not(target_os = "linux"),
        not(target_os = "android"),
        not(target_os = "ios")
    ))]
    {
        Platform::Unknown
    }
};
