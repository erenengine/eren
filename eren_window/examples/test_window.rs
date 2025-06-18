use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager};
use winit::window::Window;

struct TestWindowEventHandler {
    window: Window,
}

#[cfg(target_arch = "wasm32")]
pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    use web_sys::window;

    let window = window().expect("no global `window` exists");
    window
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

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Window) -> Self {
        console_log("Window created");
        Self { window }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        console_log(&format!("Window resized: {}x{}", width, height));
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        console_log(&format!("Scale factor changed: {}", scale_factor));
    }

    fn on_redraw_requested(&mut self) {
        console_log("Redraw requested");

        self.window.request_redraw();
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        console_log("Window lost");
    }
}

fn run() {
    match WindowLifecycleManager::<TestWindowEventHandler>::new(WindowConfig {
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
    run();
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        run();
    }
}
