use std::{collections::HashMap, sync::Arc};

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool, device::Device, instance::Instance, physical_device::PhysicalDevice,
    surface::Surface, swapchain::Swapchain,
};
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

fn load_obj_mesh(
    device: Arc<Device>,
    command_pool: &CommandPool,
    path: &std::path::Path,
) -> MeshBuffer {
    let (models, _) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
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

    MeshBuffer::new(device, command_pool, &vertices, &indices).unwrap()
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
            tex_coord: glam::Vec2::ZERO,
        })
        .collect();

    let indices: Vec<u16> = vec![0, 2, 1, 2, 0, 3];

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

        let obj_mesh = load_obj_mesh(
            device.clone(),
            &command_pool,
            "./examples/test_model/assets/viking_room.obj".as_ref(),
        );

        let (ground_vertices, ground_indices) = create_ground_plane();
        let ground_mesh = MeshBuffer::new(
            device.clone(),
            &command_pool,
            &ground_vertices,
            &ground_indices,
        )
        .unwrap();

        let meshes = vec![ground_mesh, obj_mesh];

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
