#[derive(Debug, Default)]
pub enum RenderLib {
    #[default]
    Auto,

    WGPU,

    Ash, // Vulkan
}

pub static AUTO_RENDER_LIB: RenderLib = {
    #[cfg(target_os = "windows")]
    {
        RenderLib::Ash
    }

    #[cfg(not(target_os = "windows"))]
    {
        RenderLib::WGPU
    }
};
