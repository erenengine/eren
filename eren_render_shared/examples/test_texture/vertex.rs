use glam::{Vec2, Vec3};

#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec3,
    pub tex_coords: Vec2,
}

pub const VERTEX_DESC: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
        // location(0) - Vec3 (pos)
        wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        },
        // location(1) - Vec3 (color)
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
            shader_location: 1,
            format: wgpu::VertexFormat::Float32x3,
        },
        // location(2) - Vec2 (tex_coords)
        wgpu::VertexAttribute {
            offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress * 2,
            shader_location: 2,
            format: wgpu::VertexFormat::Float32x2,
        },
    ],
};
