use thiserror::Error;

use crate::adapter::Adapter;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("Failed to request device: {0}")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
}

pub struct Device {
    handle: wgpu::Device,
    queue: wgpu::Queue,
}

impl Device {
    pub async fn new(adapter: &Adapter) -> Result<Self, DeviceError> {
        let (handle, queue) = adapter.request_device().await?;
        Ok(Self { handle, queue })
    }
}
