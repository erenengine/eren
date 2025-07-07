use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex, OnceLock},
};

use eren_render_shared::{adapter::Adapter, device::Device, instance::Instance, surface::Surface};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycle};
use glam::{Mat3, Quat, Vec2, vec2, vec3};
use winit::window::Window;

use crate::test_model::{mesh::MeshBuffer, renderer::TestRenderer, vertex::Vertex};

mod test_model {
    pub mod debug_quad_pass;
    pub mod main_pass;
    pub mod mesh;
    pub mod renderer;
    pub mod shadow_pass;
    pub mod ubo;
    pub mod vertex;
}

static IMAGE_BYTES: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();

fn set_image(bytes: Vec<u8>) {
    let cell = IMAGE_BYTES.get_or_init(|| Mutex::new(None));
    *cell.lock().unwrap() = Some(bytes);
}

fn take_image() -> Option<Vec<u8>> {
    IMAGE_BYTES.get().and_then(|m| m.lock().unwrap().take())
}

static OBJ_BYTES: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();

fn set_obj(bytes: Vec<u8>) {
    let cell = OBJ_BYTES.get_or_init(|| Mutex::new(None));
    *cell.lock().unwrap() = Some(bytes);
}

fn take_obj() -> Option<Vec<u8>> {
    OBJ_BYTES.get().and_then(|m| m.lock().unwrap().take())
}

fn load_obj_mesh(device: &Device, obj_bytes: &[u8]) -> MeshBuffer {
    let mut reader = Cursor::new(obj_bytes);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_mtl_path| Ok((Vec::new(), HashMap::new())),
    )
    .unwrap();

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut unique_vertices: HashMap<(usize, usize, usize), u16> = HashMap::new();

    let rotation = Mat3::from_quat(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2 * 3.0));
    let mesh = &models[0].mesh;

    for i in 0..mesh.indices.len() {
        // `mesh.indices` 는 position 인덱스
        let pos_idx = mesh.indices[i] as usize;

        // 만약 normals, texcoords가 있으면 그 인덱스는 `mesh.normal_indices`, `mesh.texcoord_indices` 에 있음
        let norm_idx = if !mesh.normal_indices.is_empty() {
            mesh.normal_indices[i] as usize
        } else {
            usize::MAX
        };

        let tex_idx = if !mesh.texcoord_indices.is_empty() {
            mesh.texcoord_indices[i] as usize
        } else {
            usize::MAX
        };

        let key = (pos_idx, norm_idx, tex_idx);

        if let Some(&index) = unique_vertices.get(&key) {
            indices.push(index);
            continue;
        }

        // 새 vertex 생성
        let mut position = vec3(
            mesh.positions[3 * pos_idx],
            mesh.positions[3 * pos_idx + 1],
            mesh.positions[3 * pos_idx + 2],
        );

        position = rotation * position;

        let normal = if norm_idx != usize::MAX {
            vec3(
                mesh.normals[3 * norm_idx],
                mesh.normals[3 * norm_idx + 1],
                mesh.normals[3 * norm_idx + 2],
            )
        } else {
            vec3(0.0, 1.0, 0.0)
        };

        let tex_coord = if tex_idx != usize::MAX {
            vec2(
                mesh.texcoords[2 * tex_idx],
                1.0 - mesh.texcoords[2 * tex_idx + 1],
            )
        } else {
            Vec2::ZERO
        };

        let vertex = Vertex {
            position,
            normal,
            tex_coord,
        };

        let index = vertices.len() as u16;
        vertices.push(vertex);
        indices.push(index);
        unique_vertices.insert(key, index);
    }

    log::debug!(
        "Loaded mesh with {} vertices and {} indices",
        vertices.len(),
        indices.len()
    );

    MeshBuffer::new(device, &vertices, &indices)
}

fn create_ground_plane() -> (Vec<Vertex>, Vec<u16>) {
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
            tex_coord: vec2(0.0, 0.0),
        })
        .collect();

    let indices: Vec<u16> = vec![0, 2, 1, 2, 0, 3];

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
    adapter: Adapter,
    device: Device,
    renderer: Option<TestRenderer>,

    meshes: Vec<MeshBuffer>,
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
    async fn new(window: Arc<Window>) -> Self {
        log::debug!("Window created");

        let instance = Instance::new(window.clone()).await;
        let surface = Surface::new(&instance).unwrap();
        let adapter = Adapter::new(&instance, &surface).await.unwrap();

        let window_size = window.inner_size();

        let device = Device::new(&adapter, &surface, window_size.width, window_size.height)
            .await
            .unwrap();

        let (ground_vertices, ground_indices) = create_ground_plane();
        let ground_mesh = MeshBuffer::new(&device, &ground_vertices, &ground_indices);

        let meshes = vec![ground_mesh];

        Self {
            window,
            _instance: instance,
            surface,
            adapter,
            device,
            renderer: None,

            meshes,
        }
    }

    fn on_resized(&mut self, width: u32, height: u32) {
        log::debug!("Window resized: {}x{}", width, height);

        self.device.resize_surface(&self.surface, width, height);

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(&self.device, self.adapter.depth_format, width, height);
        }
    }

    fn on_scale_factor_changed(&mut self, scale_factor: f64) {
        log::debug!("Scale factor changed: {}", scale_factor);

        let window_size = self.window.inner_size();

        self.device
            .resize_surface(&self.surface, window_size.width, window_size.height);

        if let Some(renderer) = &mut self.renderer {
            renderer.resize(
                &self.device,
                self.adapter.depth_format,
                window_size.width,
                window_size.height,
            );
        }
    }

    fn on_redraw_requested(&mut self) {
        //log::debug!("Redraw requested");

        let window_size = self.window.inner_size();

        if self.renderer.is_none()
            && let Some(image_bytes) = take_image()
            && let Some(obj_bytes) = take_obj()
        {
            log::info!(
                "Image({}) and obj({}) arrived - building renderer",
                image_bytes.len(),
                obj_bytes.len()
            );

            let obj_mesh = load_obj_mesh(&self.device, &obj_bytes);
            self.meshes.push(obj_mesh);

            self.renderer = Some(
                TestRenderer::new(
                    &self.adapter,
                    &self.device,
                    window_size.width,
                    window_size.height,
                    &image_bytes,
                )
                .expect("Failed to create renderer"),
            );

            log::debug!("Renderer created");
        }

        if let Some(renderer) = &mut self.renderer {
            renderer
                .render(
                    &self.surface,
                    &self.device,
                    &self.meshes,
                    window_size.width,
                    window_size.height,
                )
                .unwrap();
        }
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
        set_obj(std::fs::read("./examples/test_model/assets/viking_room.obj").unwrap());
        set_image(std::fs::read("./examples/test_model/assets/viking_room.png").unwrap());
        run();
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn load_obj(bytes: &[u8]) {
    set_obj(bytes.to_vec());
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn load_texture(bytes: &[u8]) {
    set_image(bytes.to_vec());
}
