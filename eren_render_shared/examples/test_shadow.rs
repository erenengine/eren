use std::sync::Arc;

use eren_render_shared::{adapter::Adapter, device::Device, instance::Instance, surface::Surface};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use winit::window::Window;

use crate::test_shadow::{mesh::MeshBuffer, renderer::TestRenderer, vertex::Vertex};

mod test_shadow {
    pub mod debug_quad_pass;
    pub mod mesh;
    pub mod renderer;
    pub mod shadow_pass;
    pub mod ubo;
    pub mod vertex;
}

fn create_ground_plane() -> (Vec<Vertex>, Vec<u16>) {
    use glam::vec3;

    // 지면은 XY 평면상에 z = 0
    // Y축이 위쪽이라면, 바닥은 -Y쪽 법선
    let normal = vec3(0.0, 1.0, 0.0);

    let positions = [
        // 두 삼각형으로 구성된 정사각형 바닥
        (vec3(-5.0, -1.0, -5.0), normal),
        (vec3(5.0, -1.0, -5.0), normal),
        (vec3(5.0, -1.0, 5.0), normal),
        (vec3(-5.0, -1.0, 5.0), normal),
    ];

    let vertices: Vec<Vertex> = positions
        .iter()
        .map(|(pos, norm)| Vertex {
            position: *pos,
            normal: *norm,
        })
        .collect();

    let indices: Vec<u16> = vec![0, 2, 1, 2, 0, 3];

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

struct TestWindowEventHandler<'a> {
    window: Arc<Window>,
    _instance: Instance,
    surface: Surface<'a>,
    _adapter: Adapter,
    device: Device,
    renderer: TestRenderer,

    meshes: Vec<MeshBuffer>,
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Instance::new(window.clone()).await;
        let surface = Surface::new(&instance).unwrap();
        let adapter = Adapter::new(&instance, &surface).await.unwrap();

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();
        let device = Device::new(
            &adapter,
            &surface,
            window_size.width / scale_factor as u32,
            window_size.height / scale_factor as u32,
        )
        .await
        .unwrap();

        let renderer = TestRenderer::new(&adapter, &device, window_size.width, window_size.height);

        log::debug!("Renderer created");

        let (ground_vertices, ground_indices) = create_ground_plane();
        let ground_mesh = MeshBuffer::new(&device, &ground_vertices, &ground_indices);

        let (cube_vertices, cube_indices) = create_cube_mesh();
        let cube_mesh = MeshBuffer::new(&device, &cube_vertices, &cube_indices);

        let meshes = vec![ground_mesh, cube_mesh];

        Self {
            window,
            _instance: instance,
            surface,
            _adapter: adapter,
            device,
            renderer,

            meshes,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);

        let scale_factor = self.window.scale_factor();
        self.device.resize_surface(
            &self.surface,
            width / scale_factor as u32,
            height / scale_factor as u32,
        );
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);

        let window_size = self.window.inner_size();
        self.device.resize_surface(
            &self.surface,
            window_size.width / scale_factor as u32,
            window_size.height / scale_factor as u32,
        );
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        self.renderer
            .render(&self.surface, &self.device, &self.meshes)
            .unwrap();
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
        run();
    }
}
