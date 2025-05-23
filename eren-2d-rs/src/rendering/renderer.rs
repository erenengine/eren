use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct Renderer {
    window: Option<Window>,
    width: u32,
    height: u32,
    title: String,
}

impl Renderer {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let mut app = Self {
            window: None,
            width,
            height,
            title: title.to_owned(),
        };

        let event_loop = EventLoop::new().unwrap();

        //event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.set_control_flow(ControlFlow::Wait);

        let _ = event_loop.run_app(&mut app);
        app
    }
}

impl ApplicationHandler for Renderer {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(LogicalSize::new(self.width, self.height));

            self.window = Some(el.create_window(attrs).unwrap());
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested if Some(id) == self.window.as_ref().map(|w| w.id()) => {
                el.exit();
            }
            _ => {}
        }
    }
}
