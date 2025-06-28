use crate::{adapter::Adapter, surface::Surface};

pub struct Device {
    handle: wgpu::Device,

    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl Device {
    pub async fn new(
        adapter: &Adapter,
        surface: &Surface<'_>,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, wgpu::RequestDeviceError> {
        let (handle, queue) = adapter.request_device().await?;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: adapter.preferred_surface_format,
            width: window_width,
            height: window_height,
            present_mode: adapter.preferred_present_mode,
            alpha_mode: adapter.preferred_alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&handle, &surface_config);

        Ok(Self {
            handle,
            queue,
            surface_config,
        })
    }

    pub fn resize_surface(&mut self, surface: &Surface<'_>, window_width: u32, window_height: u32) {
        #[cfg(not(target_os = "ios"))]
        if window_width > 0 && window_height > 0 {
            self.surface_config.width = window_width;
            self.surface_config.height = window_height;
            surface.configure(&self.handle, &self.surface_config);
        }
    }

    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.handle.create_shader_module(desc)
    }

    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.handle.create_pipeline_layout(desc)
    }

    pub fn create_render_pipeline(
        &self,
        desc: &wgpu::RenderPipelineDescriptor,
    ) -> wgpu::RenderPipeline {
        self.handle.create_render_pipeline(desc)
    }

    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.handle.create_command_encoder(desc)
    }

    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> wgpu::Buffer {
        self.handle.create_buffer(desc)
    }

    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> wgpu::BindGroupLayout {
        self.handle.create_bind_group_layout(desc)
    }

    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> wgpu::BindGroup {
        self.handle.create_bind_group(desc)
    }
}
