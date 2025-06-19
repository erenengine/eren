use crate::adapter::Adapter;

pub struct Device {
    handle: wgpu::Device,
    queue: wgpu::Queue,
}

impl Device {
    pub async fn new(adapter: &Adapter) -> Result<Self, wgpu::RequestDeviceError> {
        let (handle, queue) = adapter.request_device().await?;
        Ok(Self { handle, queue })
    }
}
