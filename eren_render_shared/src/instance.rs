use std::sync::Arc;

use thiserror::Error;
use wgpu::util::new_instance_with_webgpu_detection;
use winit::window::Window;

use crate::surface::Surface;

pub struct Instance {
    window: Arc<Window>,
    handle: wgpu::Instance,
}

impl Instance {
    pub async fn new(window: Arc<Window>) -> Self {
        //let handle = new_instance_with_webgpu_detection(&wgpu::InstanceDescriptor::default()).await;
        let handle = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
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
