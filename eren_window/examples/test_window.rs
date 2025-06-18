use std::sync::Arc;

use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use wasm_bindgen::prelude::wasm_bindgen;
use winit::window::Window;

struct TestWindowEventHandler;

#[cfg(target_arch = "wasm32")]
pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    use web_sys::window;

    let window = window().expect("no global `window` exists");
    window
        .alert_with_message(&format!("{}: {}", context, error))
        .unwrap();

    panic!("{}: {}", context, error);
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);
    }

    fn on_window_close_requested(&mut self) {
        println!("Window close requested");
    }

    fn on_redraw_requested(&mut self) {
        println!("Redraw requested");
    }
}

#[wasm_bindgen(start)]
fn start() {
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",

            #[cfg(target_arch = "wasm32")]
            canvas_id: Some("canvas"),

            #[cfg(not(target_arch = "wasm32"))]
            canvas_id: None,
        },
        TestWindowEventHandler,
    )
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

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        start();
    }
}
