use eren_render_shared::device::Device;

use super::vertex::{Vertex, create_index_buffer, create_vertex_buffer};

pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl MeshBuffer {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = create_vertex_buffer(device, vertices);
        let index_buffer = create_index_buffer(device, indices);
        Self {
            vertex_buffer,
            index_buffer,
            index_count: indices.len() as u32,
        }
    }
}
