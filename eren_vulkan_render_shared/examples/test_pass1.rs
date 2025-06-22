use std::sync::Arc;

use eren_vulkan_render_shared::{
    command::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice,
    surface::Surface, swapchain::Swapchain,
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

mod test_pass1 {
    pub mod renderer;
}

use crate::test_pass1::renderer::TestRenderer;

struct TestWindowEventHandler {
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    command_pool: Arc<CommandPool>,
    renderer: TestRenderer,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Arc::new(Instance::new(window.clone()).unwrap());
        let surface = Arc::new(Surface::new(&instance).unwrap());
        let physical_device = Arc::new(PhysicalDevice::new(instance.clone(), &surface).unwrap());
        let device = Arc::new(Device::new(instance.clone(), physical_device.clone()).unwrap());
        let swapchain = Arc::new(
            Swapchain::new(
                surface.clone(),
                &physical_device,
                device.clone(),
                window.inner_size().width,
                window.inner_size().height,
                None,
            )
            .unwrap(),
        );
        let command_pool = Arc::new(CommandPool::new(device.clone()).unwrap());
        let renderer: TestRenderer =
            TestRenderer::new(device.clone(), swapchain.clone(), command_pool.clone()).unwrap();

        log::debug!("Renderer created");

        Self {
            window,
            surface,
            physical_device,
            device,
            swapchain,
            command_pool,
            renderer,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);

        // 화면 크기 변경 시 swapchain 재생성
        let old_swapchain = self.swapchain.clone();
        self.swapchain = Arc::new(
            Swapchain::new(
                self.surface.clone(),
                &self.physical_device,
                self.device.clone(),
                width,
                height,
                Some(&old_swapchain),
            )
            .unwrap(),
        );

        // command pool과 renderer 재생성
        self.command_pool = Arc::new(CommandPool::new(self.device.clone()).unwrap());

        self.renderer = TestRenderer::new(
            self.device.clone(),
            self.swapchain.clone(),
            self.command_pool.clone(),
        )
        .unwrap();
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        self.renderer.render();

        self.window.request_redraw();
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        log::debug!("Window lost");
    }
}

pub fn main() {
    env_logger::init();

    match WindowLifecycle::<TestWindowEventHandler>::new(WindowConfig {
        width: 800,
        height: 600,
        title: "Test Window",
        canvas_id: None,
    })
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to start event loop: {}", e);
        }
    }
}
