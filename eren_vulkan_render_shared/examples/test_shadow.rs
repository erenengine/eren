use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice,
    surface::Surface, swapchain::Swapchain,
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

use crate::test_shadow::{
    mesh::{MeshBuffer, Vertex},
    renderer::TestRenderer,
};

mod test_shadow {
    pub mod debug_quad_pass;
    pub mod main_pass;
    pub mod mesh;
    pub mod renderer;
    pub mod shadow_pass;
    pub mod ubo;
}

fn create_ground_plane() -> (Vec<Vertex>, Vec<u16>) {
    use glam::vec3;

    // 지면은 XY 평면상에 z = 0
    // Y축이 위쪽이라면, 바닥은 -Y쪽 법선
    let normal = vec3(0.0, 1.0, 0.0);

    let positions = [
        // 두 삼각형으로 구성된 정사각형 바닥
        (vec3(-5.0, 0.0, -5.0), normal),
        (vec3(5.0, 0.0, -5.0), normal),
        (vec3(5.0, 0.0, 5.0), normal),
        (vec3(-5.0, 0.0, 5.0), normal),
    ];

    let vertices: Vec<Vertex> = positions
        .iter()
        .map(|(pos, norm)| Vertex {
            position: *pos,
            normal: *norm,
        })
        .collect();

    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];

    (vertices, indices)
}

fn create_cube_mesh() -> (Vec<Vertex>, Vec<u16>) {
    use glam::vec3;

    let positions = [
        // Front face
        (vec3(-0.5, -0.5, 0.5), vec3(0.0, 0.0, 1.0)),
        (vec3(0.5, -0.5, 0.5), vec3(0.0, 0.0, 1.0)),
        (vec3(0.5, 0.5, 0.5), vec3(0.0, 0.0, 1.0)),
        (vec3(-0.5, 0.5, 0.5), vec3(0.0, 0.0, 1.0)),
        // Back face
        (vec3(0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0)),
        (vec3(-0.5, -0.5, -0.5), vec3(0.0, 0.0, -1.0)),
        (vec3(-0.5, 0.5, -0.5), vec3(0.0, 0.0, -1.0)),
        (vec3(0.5, 0.5, -0.5), vec3(0.0, 0.0, -1.0)),
        // Top face
        (vec3(-0.5, 0.5, 0.5), vec3(0.0, 1.0, 0.0)),
        (vec3(0.5, 0.5, 0.5), vec3(0.0, 1.0, 0.0)),
        (vec3(0.5, 0.5, -0.5), vec3(0.0, 1.0, 0.0)),
        (vec3(-0.5, 0.5, -0.5), vec3(0.0, 1.0, 0.0)),
        // Bottom face
        (vec3(-0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0)),
        (vec3(0.5, -0.5, -0.5), vec3(0.0, -1.0, 0.0)),
        (vec3(0.5, -0.5, 0.5), vec3(0.0, -1.0, 0.0)),
        (vec3(-0.5, -0.5, 0.5), vec3(0.0, -1.0, 0.0)),
        // Right face
        (vec3(0.5, -0.5, 0.5), vec3(1.0, 0.0, 0.0)),
        (vec3(0.5, -0.5, -0.5), vec3(1.0, 0.0, 0.0)),
        (vec3(0.5, 0.5, -0.5), vec3(1.0, 0.0, 0.0)),
        (vec3(0.5, 0.5, 0.5), vec3(1.0, 0.0, 0.0)),
        // Left face
        (vec3(-0.5, -0.5, -0.5), vec3(-1.0, 0.0, 0.0)),
        (vec3(-0.5, -0.5, 0.5), vec3(-1.0, 0.0, 0.0)),
        (vec3(-0.5, 0.5, 0.5), vec3(-1.0, 0.0, 0.0)),
        (vec3(-0.5, 0.5, -0.5), vec3(-1.0, 0.0, 0.0)),
    ];

    let vertices: Vec<Vertex> = positions
        .iter()
        .map(|(pos, norm)| Vertex {
            position: *pos,
            normal: *norm,
        })
        .collect();

    let indices: Vec<u16> = vec![
        0, 1, 2, 2, 3, 0, // Front
        4, 5, 6, 6, 7, 4, // Back
        8, 9, 10, 10, 11, 8, // Top
        12, 13, 14, 14, 15, 12, // Bottom
        16, 17, 18, 18, 19, 16, // Right
        20, 21, 22, 22, 23, 20, // Left
    ];

    (vertices, indices)
}

struct TestWindowEventHandler {
    window: Arc<Window>,
    surface: Arc<Surface>,
    physical_device: Arc<PhysicalDevice>,
    device: Arc<Device>,
    command_pool: Arc<CommandPool>,
    swapchain: Arc<Swapchain>,
    renderer: TestRenderer,

    meshes: Vec<MeshBuffer>,
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
        &physical_device,
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

        let (ground_vertices, ground_indices) = create_ground_plane();
        let ground_mesh = MeshBuffer::new(
            device.clone(),
            &command_pool,
            &ground_vertices,
            &ground_indices,
        )
        .unwrap();

        let (cube_vertices, cube_indices) = create_cube_mesh();
        let cube_mesh =
            MeshBuffer::new(device.clone(), &command_pool, &cube_vertices, &cube_indices).unwrap();

        let meshes = vec![ground_mesh, cube_mesh];

        Self {
            window,
            surface,
            physical_device,
            device,
            swapchain,
            command_pool,
            renderer,
            meshes,
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

        let is_suboptimal = self.renderer.render(&self.meshes).unwrap();

        if is_suboptimal {
            let window_size = self.window.inner_size();
            self.recreate_swapchain(window_size.width, window_size.height);
        }
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
