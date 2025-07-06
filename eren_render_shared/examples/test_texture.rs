use std::sync::{Arc, Mutex, OnceLock};

use eren_render_shared::{adapter::Adapter, device::Device, instance::Instance, surface::Surface};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

mod test_texture {
    pub mod render_pass;
    pub mod renderer;
    pub mod ubo;
    pub mod vertex;
}

use crate::test_texture::renderer::TestRenderer;

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

static IMAGE_BYTES: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();

fn set_image(bytes: Vec<u8>) {
    let cell = IMAGE_BYTES.get_or_init(|| Mutex::new(None));
    *cell.lock().unwrap() = Some(bytes);
}

fn take_image() -> Option<Vec<u8>> {
    IMAGE_BYTES.get().and_then(|m| m.lock().unwrap().take())
}

struct TestWindowEventHandler<'a> {
    window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'a>,
    adapter: Adapter,
    device: Device,
    renderer: Option<TestRenderer>,
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Instance::new(window.clone()).await;
        let surface = Surface::new(&instance).unwrap();
        let adapter = Adapter::new(&instance, &surface).await.unwrap();

        let window_size = window.inner_size();
        let device = Device::new(&adapter, &surface, window_size.width, window_size.height)
            .await
            .unwrap();

        Self {
            window,
            _instance: instance,
            surface,
            adapter,
            device,
            renderer: None,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);

        self.device.resize_surface(&self.surface, width, height);

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(&self.device, self.adapter.depth_format, width, height);
        }
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);

        let window_size = self.window.inner_size();

        self.device
            .resize_surface(&self.surface, window_size.width, window_size.height);

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(
                &self.device,
                self.adapter.depth_format,
                window_size.width,
                window_size.height,
            );
        }
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        let window_size = self.window.inner_size();

        if self.renderer.is_none()
            && let Some(bytes) = take_image()
        {
            log::info!("Image arrived ({} bytes) - building renderer", bytes.len());

            self.renderer = Some(
                TestRenderer::new(
                    &self.adapter,
                    &self.device,
                    window_size.width,
                    window_size.height,
                    &bytes,
                )
                .expect("renderer create"),
            );

            log::debug!("Renderer created");
        }

        if let Some(renderer) = &mut self.renderer {
            renderer
                .render(
                    &self.surface,
                    &self.device,
                    window_size.width,
                    window_size.height,
                )
                .unwrap();
        }
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
        set_image(std::fs::read("./examples/test_texture/assets/logo.jpg").unwrap());
        run();
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn load_texture(bytes: &[u8]) {
    set_image(bytes.to_vec());
}
