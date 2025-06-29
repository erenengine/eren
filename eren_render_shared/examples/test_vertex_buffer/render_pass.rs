use eren_render_shared::device::Device;

use super::vertex::{VERTEX_DESC, Vertex};
use glam::{Vec2, Vec3};

use chrono::Utc;

const SHADER_STR: &str = include_str!("./shaders/shader.wgsl");

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1921,
    g: 0.302,
    b: 0.4745,
    a: 1.0,
};

const TEST_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: Vec2::new(-0.5, -0.5),
        color: Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, -0.5),
        color: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        pos: Vec2::new(0.5, 0.5),
        color: Vec3::new(0.0, 0.0, 1.0),
    },
    Vertex {
        pos: Vec2::new(-0.5, 0.5),
        color: Vec3::new(1.0, 1.0, 1.0),
    },
];

fn create_vertex_buffer(device: &Device) -> wgpu::Buffer {
    let vertex_size = (std::mem::size_of::<Vertex>() * TEST_VERTICES.len()) as wgpu::BufferAddress;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Buffer"),
        size: vertex_size,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_VERTICES.as_ptr() as *const u8,
            TEST_VERTICES.len() * std::mem::size_of::<Vertex>(),
        )
    };

    device.queue.write_buffer(&buffer, 0, vertex_bytes);

    buffer
}

const TEST_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

fn create_index_buffer(device: &Device) -> wgpu::Buffer {
    let index_size = (std::mem::size_of::<u16>() * TEST_INDICES.len()) as wgpu::BufferAddress;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Buffer"),
        size: index_size,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_INDICES.as_ptr() as *const u8,
            TEST_INDICES.len() * std::mem::size_of::<u16>(),
        )
    };

    device.queue.write_buffer(&buffer, 0, index_bytes);

    buffer
}

// WebGL에서는 하나의 WebGLBuffer를 gl.ARRAY_BUFFER와 gl.ELEMENT_ARRAY_BUFFER에 동시에 사용할 수 없습니다.
/*pub struct CombinedBuffer {
    pub buffer: wgpu::Buffer,
    pub vertex_offset: wgpu::BufferAddress,
    pub index_offset: wgpu::BufferAddress,
    pub index_count: u32,
}

fn create_combined_buffer(device: &Device) -> CombinedBuffer {
    let vertex_size = (std::mem::size_of::<Vertex>() * TEST_VERTICES.len()) as wgpu::BufferAddress;
    let index_size = (std::mem::size_of::<u16>() * TEST_INDICES.len()) as wgpu::BufferAddress;

    let index_offset = (vertex_size + 3) & !3;
    let total_size = index_offset + index_size;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Test Buffer"),
        size: total_size,
        usage: wgpu::BufferUsages::VERTEX
            | wgpu::BufferUsages::INDEX
            | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vertex_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_VERTICES.as_ptr() as *const u8,
            TEST_VERTICES.len() * std::mem::size_of::<Vertex>(),
        )
    };

    let index_bytes = unsafe {
        std::slice::from_raw_parts(
            TEST_INDICES.as_ptr() as *const u8,
            TEST_INDICES.len() * std::mem::size_of::<u16>(),
        )
    };

    device.queue.write_buffer(&buffer, 0, vertex_bytes);
    device
        .queue
        .write_buffer(&buffer, index_offset, index_bytes);

    CombinedBuffer {
        buffer,
        vertex_offset: 0,
        index_offset,
        index_count: TEST_INDICES.len() as u32,
    }
}*/

pub struct TestRenderPass {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

impl TestRenderPass {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Test Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_STR.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[VERTEX_DESC],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: device.surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = create_vertex_buffer(&device);
        let index_buffer = create_index_buffer(&device);

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            index_count: TEST_INDICES.len() as u32,
        }
    }

    pub fn record_commands(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Test Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..));
        render_pass.set_index_buffer(self.index_buffer.slice(0..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
