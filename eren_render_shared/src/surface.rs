use std::sync::Arc;

use thiserror::Error;

use crate::instance::Instance;

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("Failed to create surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
}

pub struct Surface<'window> {
    handle: wgpu::Surface<'window>,
}

impl<'window> Surface<'window> {
    pub fn new(instance: &Instance) -> Result<Self, SurfaceError> {
        let handle = instance.create_surface()?;
        Ok(Self { handle })
    }

    pub fn get_compatible_surface(&self) -> &wgpu::Surface {
        &self.handle
    }
}
