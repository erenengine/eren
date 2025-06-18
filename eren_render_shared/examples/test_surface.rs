use std::sync::Arc;

use eren_render_shared::{instance::Instance, surface::Surface};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    web_sys::window()
        .unwrap()
        .alert_with_message(&format!("{}: {}", context, error))
        .unwrap();

    panic!("{}: {}", context, error);
}

fn console_log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&message.into());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("{}", message);
    }
}

struct TestWindowEventHandler<'window> {
    window: Arc<Window>,
    instance: Instance,
    surface: Surface<'window>,
}

impl<'window> WindowEventHandler for TestWindowEventHandler<'window> {
    async fn new(window: Arc<Window>) -> Self {
        console_log("Window created");

        let instance = Instance::new(window.clone()).await;
        let surface = Surface::new(&instance).unwrap();

        console_log("Surface created");

        Self {
            window,
            instance,
            surface,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        console_log(&format!("Window resized: {}x{}", width, height));
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        console_log(&format!("Scale factor changed: {}", scale_factor));
    }

    fn on_redraw_requested(&mut self) {
        //console_log("Redraw requested");

        self.window.request_redraw();
    }
}

impl<'a> Drop for TestWindowEventHandler<'a> {
    fn drop(&mut self) {
        console_log("Window lost");
    }
}

fn run() {
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
        Err(e) => {
            #[cfg(target_arch = "wasm32")]
            {
                show_error_popup_and_panic(e, "Failed to start event loop");
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                eprintln!("Failed to start event loop: {}", e);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
    run();
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run();
    }
}
