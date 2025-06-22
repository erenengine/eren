use std::sync::Arc;

use eren_vulkan_render_shared::{
    device::Device, instance::Instance, physical_device::PhysicalDevice, surface::Surface,
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

mod test_pass1 {
    pub mod renderer;
}

use crate::test_pass1::renderer::TestRenderer;

struct TestWindowEventHandler {
    window: Arc<Window>,
    _instance: Arc<Instance>,
    _surface: Surface,
    _physical_device: Arc<PhysicalDevice>,
    _device: Arc<Device>,
    renderer: TestRenderer,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Arc::new(Instance::new(window.clone()).unwrap());
        let surface = Surface::new(&instance).unwrap();
        let physical_device = Arc::new(PhysicalDevice::new(instance.clone(), &surface).unwrap());
        let device = Arc::new(Device::new(physical_device.clone()).unwrap());
        let renderer: TestRenderer = TestRenderer::new(device.clone()).unwrap();

        log::debug!("Renderer created");

        Self {
            window,
            _instance: instance,
            _surface: surface,
            _physical_device: physical_device,
            _device: device.clone(),
            renderer,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);
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
