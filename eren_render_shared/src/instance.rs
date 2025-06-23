use std::sync::Arc;

use wgpu::util::new_instance_with_webgpu_detection;
use winit::window::Window;

pub struct Instance {
    window: Arc<Window>,
    handle: wgpu::Instance,
}

impl Instance {
    pub async fn new(window: Arc<Window>) -> Self {
        let instance_desc = wgpu::InstanceDescriptor {
            //backends: wgpu::Backends::GL, // 오직 WebGL만 대상으로 함
            ..Default::default()
        };

        let handle = new_instance_with_webgpu_detection(&instance_desc).await;
        Self { window, handle }
    }

    pub fn create_surface<'a>(&self) -> Result<wgpu::Surface<'a>, wgpu::CreateSurfaceError> {
        self.handle.create_surface(self.window.clone())
    }

    pub async fn request_adapter<'a>(
        &self,
        compatible_surface: Option<&wgpu::Surface<'a>>,
    ) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
        self.handle
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await
    }
}
