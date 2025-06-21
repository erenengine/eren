use std::sync::Arc;

use ash::vk::CommandBuffer;
use eren_vulkan_render_shared::device::Device;
use glam::{Mat4, Vec3};

pub struct Camera {
    pub view_proj_matrix: [[f32; 4]; 4],
}

pub struct DirectionalLight {
    /// 빛의 방향 (단위 벡터, 월드 좌표 기준)
    pub direction: [f32; 3],

    /// 강도 (조도 조절용 스칼라값)
    pub intensity: f32,

    /// 빛의 색 (RGB, 보통 값은 0.0~1.0)
    pub color: [f32; 4],
}

pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct TestRenderView {
    meshes: Vec<Mesh>,
}

pub struct TestRenderer {
    camera: Camera,
    light: DirectionalLight,
    views: Vec<TestRenderView>,
}

impl TestRenderer {
    pub fn new(_device: Arc<Device>) -> Self {
        let eye = Vec3::new(4.0, -6.0, 5.0); // (x, y, z)
        let centre = Vec3::ZERO; // look at sphere at world origin
        let up = Vec3::Z; // z‑axis is "up" in this world
        let view_mat = Mat4::look_at_rh(eye, centre, up);
        let proj_mat = Mat4::perspective_rh_gl(60f32.to_radians(), 1.0, 0.1, 100.0);
        let view_proj = proj_mat * view_mat;

        let camera = Camera {
            view_proj_matrix: view_proj.to_cols_array_2d(),
        };

        // --------------------------------------------------
        // 3. Sun / directional light placement
        // --------------------------------------------------
        // Aim the light so it comes from the top‑right‑back corner.
        // That means light direction (towards the sun) ≈ (-0.7, 0.5, -1.0).
        let raw_dir = Vec3::new(-0.7, 0.5, -1.0).normalize();
        let light = DirectionalLight {
            direction: raw_dir.into(), // Into <[f32;3]>
            intensity: 4.0,
            color: [1.0, 0.98, 0.92, 1.0], // Slightly warm sunlight
        };

        let sphere_mesh = generate_uv_sphere(1.0, 32, 16);

        let view = TestRenderView {
            meshes: vec![
                // Plane
                Mesh {
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
                    indices: vec![
                        0, 1, 2, // Triangle 1
                        2, 1, 3, // Triangle 2
                    ],
                },
                // Sphere
                sphere_mesh,
            ],
        };

        Self {
            camera,
            light,
            views: vec![view],
        }
    }

    pub fn render(&self, _command_buffer: &mut CommandBuffer) {
        // TODO: Rendering implementation goes here.
    }
}

pub mod render_passes;

/// Generates a UV sphere and returns it as a `Mesh`.
fn generate_uv_sphere(radius: f32, lon: u32, lat: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for y in 0..=lat {
        let v = y as f32 / lat as f32;
        let theta = v * std::f32::consts::PI;

        for x in 0..=lon {
            let u = x as f32 / lon as f32;
            let phi = u * std::f32::consts::TAU;

            let pos = [
                radius * phi.sin() * theta.sin(),
                radius * theta.cos(),
                radius * phi.cos() * theta.sin(),
            ];

            // Normal is the normalized position vector for a sphere centered at the origin
            let normal = [pos[0] / radius, pos[1] / radius, pos[2] / radius];

            vertices.push(Vertex {
                position: pos,
                normal,
            });
        }
    }

    // Generate indices
    for y in 0..lat {
        for x in 0..lon {
            let i0 = y * (lon + 1) + x;
            let i1 = i0 + lon + 1;

            indices.extend_from_slice(&[i0, i1, i0 + 1, i0 + 1, i1, i1 + 1]);
        }
    }

    Mesh { vertices, indices }
}
