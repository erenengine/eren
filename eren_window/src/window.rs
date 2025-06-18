use std::sync::Arc;

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::StartCause,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::Window,
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub trait WindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) -> impl Future<Output = ()> + Send; // 윈도우가 생성되었을 때 -> GPU 리소스 할당

    fn on_window_lost(&mut self); // 윈도우가 없어졌을 때 -> GPU 리소스 해제

    fn on_window_resized(&mut self, size: WindowSize); // 윈도우 크기가 변경되었을 때 -> 서피스 재설정

    fn on_window_close_requested(&mut self); // 윈도우가 닫히는 요청을 받았을 때 -> GPU 리소스 해제

    fn on_redraw_requested(&mut self); // 윈도우가 다시 그려질 때 -> GPU에 그리기
}

#[derive(Debug, Error)]
pub enum WindowLifecycleManagerError {
    #[error("Event loop error: {0}")]
    EventLoopError(#[from] EventLoopError),
}

#[derive(Debug)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub canvas_id: Option<&'static str>,
}

pub struct WindowLifecycleManager<E: WindowEventHandler + 'static> {
    config: WindowConfig,
    event_handler: E,

    proxy: Option<EventLoopProxy<E>>,
    window: Option<Arc<Window>>,
    window_size: Option<WindowSize>,
}

impl<E: WindowEventHandler> WindowLifecycleManager<E> {
    pub fn new(config: WindowConfig, event_handler: E) -> Self {
        Self {
            config,
            event_handler,

            proxy: None,
            window: None,
            window_size: None,
        }
    }

    fn handle_resize_event(&mut self, new_size: WindowSize) {
        // (0,0)은 일부 플랫폼에서 초기화 과정에서 나오는 값이므로 무시
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        if self.window_size != Some(new_size) {
            self.window_size = Some(new_size);
            self.event_handler.on_window_resized(new_size);
        }
    }
}

impl<E: WindowEventHandler> WindowLifecycleManager<E> {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_event_loop(&mut self) -> Result<(), WindowLifecycleManagerError> {
        use winit::event_loop::ControlFlow;

        let event_loop = EventLoop::<E>::with_user_event().build()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;

        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn start_event_loop(self) -> Result<(), WindowLifecycleManagerError> {
        use winit::platform::web::EventLoopExtWebSys;

        let event_loop = EventLoop::<E>::with_user_event().build()?;

        // self를 move하기 전에 proxy 설정
        let mut manager = self;
        manager.proxy = Some(event_loop.create_proxy());

        event_loop.spawn_app(manager);

        Ok(())
    }
}

impl<E: WindowEventHandler> ApplicationHandler<E> for WindowLifecycleManager<E> {
    #[cfg(not(target_arch = "wasm32"))]
    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if let Some(window) = &self.window {
            if let StartCause::Poll = cause {
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_some() {
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
            pollster::block_on(self.event_handler.on_window_ready(window.clone()));
        }

        //#[cfg(target_arch = "wasm32")]
        {
            // future를 비동기적으로 실행하고
            // proxy를 사용해 결과를 이벤트 루프로 보냅니다
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
    }
}
