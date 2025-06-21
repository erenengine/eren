use std::sync::Arc;

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoopProxy;

pub trait WindowEventHandler {
    // 윈도우가 생성되었을 때 -> GPU 리소스 할당
    fn new(window: Arc<Window>) -> impl Future<Output = Self>;

    // 윈도우 크기가 변경되었을 때 -> 서피스 재설정
    fn on_resized(&mut self, width: u32, height: u32);

    // 윈도우의 scale factor가 변경되었을 때 -> 서피스 재설정
    fn on_scale_factor_changed(&mut self, scale_factor: f64);

    // 윈도우가 다시 그려질 때 -> GPU에 그리기
    fn on_redraw_requested(&mut self);
}

#[derive(Debug, Error)]
pub enum WindowLifecycleError {
    #[error("Window lifecycle error: {0}")]
    EventLoopError(#[from] EventLoopError),
}

#[derive(Debug)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub canvas_id: Option<&'static str>,
}

pub struct WindowLifecycle<E: WindowEventHandler + 'static> {
    config: WindowConfig,
    event_handler: Option<E>,

    #[cfg(target_arch = "wasm32")]
    proxy: Option<EventLoopProxy<E>>,
}

impl<E: WindowEventHandler> WindowLifecycle<E> {
    pub fn new(config: WindowConfig) -> Self {
        Self {
            config,
            event_handler: None,

            #[cfg(target_arch = "wasm32")]
            proxy: None,
        }
    }
}

impl<E: WindowEventHandler> WindowLifecycle<E> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_event_loop(&mut self) -> Result<(), WindowLifecycleError> {
        let event_loop = EventLoop::<E>::with_user_event().build()?;
        event_loop.run_app(self)?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_event_loop(self) -> Result<(), WindowLifecycleError> {
        use winit::platform::web::EventLoopExtWebSys;

        let event_loop = EventLoop::<E>::with_user_event().build()?;

        // self를 move하기 전에 proxy 설정
        let mut manager = self;
        manager.proxy = Some(event_loop.create_proxy());

        event_loop.spawn_app(manager);
        Ok(())
    }
}

impl<E: WindowEventHandler> ApplicationHandler<E> for WindowLifecycle<E> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.event_handler.is_some() {
            return;
        }

        let raw_window = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                use winit::dpi::LogicalSize;

                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(self.config.title)
                            .with_inner_size(LogicalSize::new(
                                self.config.width,
                                self.config.height,
                            )),
                    )
                    .expect("Failed to create native window")
            }

            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::JsCast;
                use web_sys::HtmlCanvasElement;
                use winit::platform::web::WindowAttributesExtWebSys;

                // Look up the target <canvas> from the HTML document.
                let canvas: HtmlCanvasElement = {
                    let window = web_sys::window().expect("No global `window`");
                    let document = window.document().expect("No Document");
                    document
                        .get_element_by_id(self.config.canvas_id.expect("Canvas ID is not set"))
                        .unwrap_or_else(|| {
                            panic!(
                                "Canvas element #{} not found",
                                self.config.canvas_id.expect("Canvas ID is not set")
                            )
                        })
                        .dyn_into::<HtmlCanvasElement>()
                        .expect("Element is not a canvas")
                };

                event_loop
                    .create_window(Window::default_attributes().with_canvas(Some(canvas)))
                    .expect("Failed to create web window")
            }
        };

        let window = Arc::new(raw_window);

        #[cfg(not(target_arch = "wasm32"))]
        {
            // 웹 환경이 아니라면 pollster를 사용하여
            // future를 동기적으로 기다릴 수 있습니다
            self.event_handler = Some(pollster::block_on(E::new(window.clone())));

            window.request_redraw();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // future를 비동기적으로 실행하고
            // proxy를 사용해 결과를 이벤트 루프로 보냅니다
            if let Some(proxy) = self.proxy.take() {
                let cloned_window = window.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let event_handler = E::new(cloned_window.clone()).await;
                    //cloned_window.request_redraw();
                    assert!(proxy.send_event(event_handler).is_ok());
                });
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event_handler: E) {
        // proxy.send_event()가 보낸 이벤트가 여기로 도착합니다
        self.event_handler = Some(event_handler);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let event_handler = match &mut self.event_handler {
            Some(event_handler) => event_handler,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => {
                event_handler.on_resized(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                self.event_handler = None;
                event_loop.exit();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                event_handler.on_scale_factor_changed(scale_factor);
            }
            WindowEvent::RedrawRequested => {
                event_handler.on_redraw_requested();
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.event_handler = None;
    }
}
