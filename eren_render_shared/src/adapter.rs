use crate::{instance::Instance, surface::Surface};

pub struct Adapter {
    handle: wgpu::Adapter,
}

impl Adapter {
    pub async fn new<'window>(
        instance: &Instance,
        surface: &Surface<'window>,
    ) -> Result<Self, wgpu::RequestAdapterError> {
        let compatible_surface = Some(surface.get_compatible_surface());
        let handle = instance.request_adapter(compatible_surface).await?;
        Ok(Self { handle })
    }

    pub async fn request_device(
        &self,
    ) -> Result<(wgpu::Device, wgpu::Queue), wgpu::RequestDeviceError> {
        let mut desc = wgpu::DeviceDescriptor::default();
        desc.required_limits = self.handle.limits();
        self.handle.request_device(&desc).await
    }
}
