use std::sync::Arc;

use eren_vulkan_render_shared::instance::Instance;
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

struct TestWindowEventHandler {
    window: Arc<Window>,
    _instance: Instance,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Instance::new(window.clone()).unwrap();

        log::debug!("Instance created");

        Self {
            window,
            _instance: instance,
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
