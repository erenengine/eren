use std::sync::Arc;

use eren_render_shared::{adapter::Adapter, device::Device, instance::Instance, surface::Surface};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

mod test_pass1 {
    pub mod renderer;
}

use crate::test_pass1::renderer::TestRenderer;

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

struct TestWindowEventHandler<'a> {
    window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'a>,
    _adapter: Adapter,
    device: Device,
    renderer: TestRenderer,
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Instance::new(window.clone()).await;
        let surface = Surface::new(&instance).unwrap();
        let adapter = Adapter::new(&instance, &surface).await.unwrap();

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let device = Device::new(
            &adapter,
            &surface,
            window_size.width / scale_factor as u32,
            window_size.height / scale_factor as u32,
        )
        .await
        .unwrap();

        let renderer = TestRenderer::new(&device);

        log::debug!("Renderer created");

        Self {
            window,
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            renderer,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);

        let scale_factor = self.window.scale_factor();
        self.device.resize_surface(
            &self.surface,
            width / scale_factor as u32,
            height / scale_factor as u32,
        );
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);

        let window_size = self.window.inner_size();
        self.device.resize_surface(
            &self.surface,
            window_size.width / scale_factor as u32,
            window_size.height / scale_factor as u32,
        );
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        self.renderer.render(&self.surface, &self.device).unwrap();

        self.window.request_redraw();
    }
}

impl<'a> Drop for TestWindowEventHandler<'a> {
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
