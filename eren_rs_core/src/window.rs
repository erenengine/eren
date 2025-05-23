use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct WindowState {
    window: Option<Arc<Window>>,
    sender: Option<oneshot::Sender<Arc<Window>>>,
    width: u32,
    height: u32,
    title: String,
}

impl ApplicationHandler for WindowState {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(LogicalSize::new(self.width, self.height));

            let window = Arc::new(el.create_window(attrs).unwrap());
            if let Some(sender) = self.sender.take() {
                let _ = sender.send(window.clone());
            }
            self.window = Some(window);
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            if Some(id) == self.window.as_ref().map(|w| w.id()) {
                el.exit();
            }
        }
    }
}

pub async fn create_window(width: u32, height: u32, title: &str) -> Arc<Window> {
    let (tx, rx) = oneshot::channel();
    let app = Arc::new(Mutex::new(WindowState {
        window: None,
        sender: Some(tx),
        width,
        height,
        title: title.to_owned(),
    }));

    let app_clone = app.clone();
    std::thread::spawn(move || {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Wait);
        let _ = event_loop.run_app(&mut *app_clone.lock().unwrap());
    });

    rx.await.expect("Window creation failed")
}
