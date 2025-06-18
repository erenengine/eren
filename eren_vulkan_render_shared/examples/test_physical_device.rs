use std::sync::Arc;

use eren_render_vulkan_shared::{instance::Instance, physical_device::PhysicalDevice};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

struct TestWindowEventHandler {
    window: Arc<Window>,
    instance: Instance,
    physical_device: PhysicalDevice,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        println!("Window created");

        let instance = Instance::new(window.clone()).unwrap();
        let physical_device = PhysicalDevice::new(&instance).unwrap();

        println!("Physical device created");

        Self {
            window,
            instance,
            physical_device,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        println!("Window resized: {}x{}", width, height);
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        println!("Scale factor changed: {}", scale_factor);
    }

    fn on_redraw_requested(&mut self) {
        //println!("Redraw requested");

        self.window.request_redraw();
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        println!("Window lost");
    }
}

pub fn main() {
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
            eprintln!("Failed to start event loop: {}", e);
        }
    }
}
