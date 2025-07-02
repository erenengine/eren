use std::sync::Arc;

use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

pub fn init_logger() {
    #[cfg(target_arch = "wasm32")]
    {
        use log::Level;

        console_error_panic_hook::set_once();
        console_log::init_with_level(Level::Debug).expect("Failed to init console_log");
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
}

struct TestWindowEventHandler {
    _window: Arc<Window>,
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");
        Self { _window: window }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);
    }

    fn on_redraw_requested(&mut self) {
        log::debug!("Redraw requested");
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        log::debug!("Window lost");
    }
}

fn run() {
    init_logger();

    match WindowLifecycle::<TestWindowEventHandler>::new(WindowConfig {
        width: 800,
        height: 600,
        title: "Test Window",

        #[cfg(target_arch = "wasm32")]
        canvas_id: Some("canvas"),

        #[cfg(not(target_arch = "wasm32"))]
        canvas_id: None,
    })
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => log::error!("Failed to start event loop: {}", e),
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    run();
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run();
    }
}
