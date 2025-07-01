use crate::{instance::Instance, surface::Surface};

pub struct Adapter {
    handle: wgpu::Adapter,

    pub preferred_surface_format: wgpu::TextureFormat,
    pub preferred_present_mode: wgpu::PresentMode,
    pub preferred_alpha_mode: wgpu::CompositeAlphaMode,
}

fn select_preferred_surface_format(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
    log::debug!("Available formats: {:#?}", formats);
    formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        // sRGB가 없으면 첫 번째(플랫폼이 이미 추천한) 형식 사용
        .unwrap_or_else(|| formats[0])
}

impl Adapter {
    pub async fn new<'window>(
        instance: &Instance,
        surface: &Surface<'window>,
    ) -> Result<Self, wgpu::RequestAdapterError> {
        let compatible_surface = Some(surface.get_compatible_surface());
        let handle = instance.request_adapter(compatible_surface).await?;

        let surface_caps = surface.get_capabilities(&handle);
        let preferred_surface_format = select_preferred_surface_format(&surface_caps.formats);
        let preferred_present_mode = surface_caps.present_modes[0];
        let preferred_alpha_mode = surface_caps.alpha_modes[0];

        Ok(Self {
            handle,

            preferred_surface_format,
            preferred_present_mode,
            preferred_alpha_mode,
        })
    }

    pub async fn request_device(
        &self,
    ) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        // WebGPU/WebGL에서 push constants를 지원하지 않음
        //let required_features = wgpu::Features::PUSH_CONSTANTS;

        // WebGPU에서는 read_write storage buffer를 버텍스 셰이더에서 사용할 수 없음
        // WebGL에서는 storage buffer를 사용할 수 없음
        //let required_features = wgpu::Features::VERTEX_WRITABLE_STORAGE;

        let mut desc = wgpu::DeviceDescriptor::default();
        //desc.required_features = required_features;
        desc.required_limits = self.handle.limits();
        self.handle.request_device(&desc).await
    }
}
