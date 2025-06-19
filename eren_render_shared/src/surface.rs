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
}
