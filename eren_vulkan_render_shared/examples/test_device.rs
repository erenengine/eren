use std::sync::Arc;

use eren_render_vulkan_shared::{
    device::Device, instance::Instance, physical_device::PhysicalDevice, surface::Surface,
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

struct TestWindowEventHandler {
    window: Arc<Window>,
    instance: Option<Arc<Instance>>,
    surface: Option<Surface>,
    physical_device: Option<PhysicalDevice>,
    device: Option<Device>,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Arc::new(Instance::new(window.clone()).unwrap());
        let surface = Surface::new(instance.clone()).unwrap();
        let physical_device = PhysicalDevice::new(instance.clone(), &surface).unwrap();
        let device = Device::new(&physical_device).unwrap();

        log::debug!("Device created");

        Self {
            window,
            instance: Some(instance),
            surface: Some(surface),
            physical_device: Some(physical_device),
            device: Some(device),
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

        self.window.request_redraw();
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        log::debug!("Window lost");

        // Drop in reverse order
        self.device = None;
        self.physical_device = None;
        self.surface = None;
        self.instance = None;
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
