use std::sync::Arc;

use ash::vk::CommandBuffer;
use eren_vulkan_render_shared::device::Device;
use glam::{Mat4, Vec3};

/// Simple right‑handed, Z‑up scene with a plane and a sphere.
/// The code prepares camera & light matrices compatible with Vulkan (NDC z ∈ 0‥1).
/// Shadow/scene passes are expected to be recorded elsewhere.

// ─────────────────────────────────────────────────────────────────────────────
//  Data structures sent to GPU
// ─────────────────────────────────────────────────────────────────────────────

pub struct Camera {
    /// Projection × View matrix (column‑major)
    pub view_proj_matrix: [[f32; 4]; 4],
}

pub struct DirectionalLight {
    /// **Surface → Light** unit vector (world space)
    pub direction: [f32; 3],
    /// Scalar multiplier for radiance
    pub intensity: f32,
    /// RGBA linear color (0..1)
    pub color: [f32; 4],
}

#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

/// Geometry + model matrix bundled for easy iteration
pub struct Renderable {
    pub mesh: Mesh,
    pub model_matrix: [[f32; 4]; 4],
}

pub struct TestRenderView {
    pub objects: Vec<Renderable>,
}

pub struct TestRenderer {
    camera: Camera,
    light: DirectionalLight,
    view: TestRenderView,
}

impl TestRenderer {
    /// Builds a plane (XY @ Z=0) and a unit sphere resting on it.
    /// `Device` is kept for later pipeline/resource creation.
    pub fn new(_device: Arc<Device>) -> Self {
        // ────────────────────── 1. Camera ──────────────────────
        let eye = Vec3::new(4.0, -6.0, 5.0);
        let centre = Vec3::ZERO;
        let up = Vec3::Z; // Z‑up world

        let view_mat = Mat4::look_at_rh(eye, centre, up);
        // Vulkan NDC expects depth 0‥1, so use perspective_rh()
        let aspect = 1.0_f32; // Swapchain ratio; update later
        let proj_mat = Mat4::perspective_rh(60f32.to_radians(), aspect, 0.1, 100.0);

        let camera = Camera {
            view_proj_matrix: (proj_mat * view_mat).to_cols_array_2d(),
        };

        // ────────────────────── 2. Light ──────────────────────
        // Surface → Sun direction
        let raw_dir = Vec3::new(-0.7, 0.5, -1.0).normalize();
        let light = DirectionalLight {
            direction: raw_dir.into(),
            intensity: 4.0,
            color: [1.0, 0.98, 0.92, 1.0], // Warmish daylight
        };

        // ────────────────────── 3. Geometry ──────────────────────
        // 3‑A. Infinite ground (quad)
        let plane_mesh = Mesh {
            vertices: vec![
                Vertex {
                    position: [-100.0, -100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [100.0, -100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [-100.0, 100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
                Vertex {
                    position: [100.0, 100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 2, 1, 3],
        };

        // 3‑B. Sphere (radius 1) translated so it sits on the plane
        let sphere_mesh = generate_uv_sphere_z_up(1.0, 32, 16);
        let sphere_model = Mat4::from_translation(Vec3::Z); // (0, 0, 1)

        let view = TestRenderView {
            objects: vec![
                Renderable {
                    mesh: plane_mesh,
                    model_matrix: Mat4::IDENTITY.to_cols_array_2d(),
                },
                Renderable {
                    mesh: sphere_mesh,
                    model_matrix: sphere_model.to_cols_array_2d(),
                },
            ],
        };

        Self {
            camera,
            light,
            view,
        }
    }

    /// Records shadow & scene passes. Actual pipeline setup omitted.
    pub fn render(&self) {
        // TODO ────────────────────────────────────────────────────────────
        // 1. Shadow pass: draw each object with lightViewProj to depth only
        // 2. Barrier → read‑only, then scene pass: draw with camera VP, shadow map
        // Resources (uniform buffers / push constants):
        //   * Camera::view_proj_matrix
        //   * Model matrix per‑object
        //   * DirectionalLight (direction, intensity, color)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Helper: UV sphere (Z‑up) ---------------------------------------------------
// ─────────────────────────────────────────────────────────────────────────────

fn generate_uv_sphere_z_up(radius: f32, lon: u32, lat: u32) -> Mesh {
    let mut vertices = Vec::with_capacity(((lon + 1) * (lat + 1)) as usize);
    let mut indices = Vec::with_capacity((lon * lat * 6) as usize);

    for y in 0..=lat {
        let v = y as f32 / lat as f32; // 0..1
        let theta = v * std::f32::consts::PI; // 0..π

        for x in 0..=lon {
            let u = x as f32 / lon as f32; // 0..1
            let phi = u * std::f32::consts::TAU; // 0..2π

            let pos = Vec3::new(
                radius * theta.sin() * phi.cos(),
                radius * theta.sin() * phi.sin(),
                radius * theta.cos(), // Z‑up
            );

            vertices.push(Vertex {
                position: pos.into(),
                normal: (pos / radius).into(),
            });
        }
    }

    for y in 0..lat {
        for x in 0..lon {
            let i0 = y * (lon + 1) + x;
            let i1 = i0 + lon + 1;
            indices.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]);
        }
    }

    Mesh { vertices, indices }
}
