use crate::instance::Instance;

pub struct Surface<'window> {
    handle: wgpu::Surface<'window>,
}

impl<'window> Surface<'window> {
    pub fn new(instance: &Instance) -> Result<Self, wgpu::CreateSurfaceError> {
        let handle = instance.create_surface()?;
        Ok(Self { handle })
    }

    pub fn get_compatible_surface(&self) -> &wgpu::Surface {
        &self.handle
    }

    pub fn get_capabilities(&self, adapter: &wgpu::Adapter) -> wgpu::SurfaceCapabilities {
        self.handle.get_capabilities(adapter)
    }

    pub fn configure(&self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
        self.handle.configure(device, config);
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.handle.get_current_texture()
    }
}
