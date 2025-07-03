use eren_render_shared::device::Device;
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3, // layout(location = 0)
    pub normal: Vec3,   // layout(location = 1)
}

pub const VERTEX_DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        // location(0) - Vec3 (position)
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        // location(1) - Vec3 (normal)
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3,
        },
    ],
};

pub fn create_vertex_buffer(device: &Device, vertices: &[Vertex]) -> wgpu::Buffer {
    let vertex_size = (std::mem::size_of::<Vertex>() * vertices.len()) as wgpu::BufferAddress;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Vertex Buffer"),
        size: vertex_size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<Vertex>(),
        )
    };

    device.queue.write_buffer(&buffer, 0, vertex_bytes);

    buffer
}

pub fn create_index_buffer(device: &Device, indices: &[u16]) -> wgpu::Buffer {
    let index_size = (std::mem::size_of::<u16>() * indices.len()) as wgpu::BufferAddress;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Index Buffer"),
        size: index_size,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr() as *const u8,
            indices.len() * std::mem::size_of::<u16>(),
        )
    };

    device.queue.write_buffer(&buffer, 0, index_bytes);

    buffer
}
