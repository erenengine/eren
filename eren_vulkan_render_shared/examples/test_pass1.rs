use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice,
    surface::Surface, swapchain::Swapchain,
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

mod test_pass1 {
    pub mod renderer;
}

use crate::test_pass1::renderer::TestRenderer;

struct TestWindowEventHandler {
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    swapchain: Arc<Swapchain>,
    renderer: TestRenderer,
}

fn create_swapchain(
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    old_swapchain: Option<&Swapchain>,
    width: u32,
    height: u32,
) -> (Arc<Swapchain>, TestRenderer) {
    // 화면 크기 변경 시 swapchain 재생성
    let swapchain = Arc::new(
        Swapchain::new(
            surface,
            &physical_device,
            device.clone(),
            width,
            height,
            old_swapchain,
        )
        .unwrap(),
    );

    // renderer 재생성

    let renderer = TestRenderer::new(
        device,
        swapchain.clone(),
        &command_pool,
        vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D { width, height },
        },
    )
    .unwrap();

    (swapchain, renderer)
}

impl TestWindowEventHandler {
    fn recreate_swapchain(&mut self, width: u32, height: u32) {
        let (swapchain, renderer) = create_swapchain(
            self.surface.clone(),
            self.physical_device.clone(),
            self.device.clone(),
            self.command_pool.clone(),
            Some(&self.swapchain),
            width,
            height,
        );

        self.swapchain = swapchain;
        self.renderer = renderer;
    }
}

impl WindowEventHandler for TestWindowEventHandler {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Arc::new(Instance::new(window.clone()).unwrap());
        let surface = Arc::new(Surface::new(instance.clone()).unwrap());
        let physical_device =
            Arc::new(PhysicalDevice::new(instance.clone(), surface.clone()).unwrap());
        let device = Arc::new(Device::new(instance.clone(), physical_device.clone()).unwrap());
        let command_pool = Arc::new(CommandPool::new(device.clone()).unwrap());

        let window_size = window.inner_size();
        let (swapchain, renderer) = create_swapchain(
            surface.clone(),
            physical_device.clone(),
            device.clone(),
            command_pool.clone(),
            None,
            window_size.width,
            window_size.height,
        );

        log::debug!("Renderer created");

        let window_scale_factor = window.scale_factor();
        log::debug!("Window scale factor: {}", window_scale_factor);

        Self {
            window,
            surface,
            physical_device,
            device,
            swapchain,
            command_pool,
            renderer,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);
        self.recreate_swapchain(width, height);
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);

        //TODO: 테스트해보기
        /*let window_size = self.window.inner_size();
        self.recreate_swapchain(window_size.width, window_size.height);*/
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        let is_suboptimal = self.renderer.render().unwrap();

        if is_suboptimal {
            let window_size = self.window.inner_size();
            self.recreate_swapchain(window_size.width, window_size.height);
        }

        self.window.request_redraw();
    }
}

impl Drop for TestWindowEventHandler {
    fn drop(&mut self) {
        log::debug!("Window lost");
    }
}

pub fn main() {
    env_logger::init();

    match WindowLifecycle::<TestWindowEventHandler>::new(WindowConfig {
        width: 800,
        height: 600,
        title: "Test Window",
        canvas_id: None,
    })
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to start event loop: {}", e);
        }
    }
}
