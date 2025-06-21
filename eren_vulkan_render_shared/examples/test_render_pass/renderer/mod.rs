use std::sync::Arc;

use ash::vk::CommandBuffer;
use eren_vulkan_render_shared::device::Device;

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
    views: Vec<TestRenderView>,
}

impl TestRenderer {
    pub fn new(_device: Arc<Device>) -> Self {
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
        Self { views: vec![view] }
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
