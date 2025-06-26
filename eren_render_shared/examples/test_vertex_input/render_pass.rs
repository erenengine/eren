use eren_render_shared::device::Device;

use crate::test_vertex_input::{ubo::UniformBufferObject, vertex::Vertex};
use glam::{Vec2, Vec3};

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

const TEST_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct CombinedBuffer {
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
}

pub struct TestRenderPass {
    pipeline: wgpu::RenderPipeline,
    combined_buffer: CombinedBuffer,
    ubo_buffer: wgpu::Buffer,
    ubo_bind_group: wgpu::BindGroup,
    start_time: std::time::Instant,
}

impl TestRenderPass {
    pub fn new(device: &Device) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Test Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_STR.into()),
        });

        let ubo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UBO Buffer"),
            size: std::mem::size_of::<UniformBufferObject>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let ubo_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("UBO Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let ubo_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UBO Bind Group"),
            layout: &ubo_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ubo_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Test Pipeline Layout"),
            bind_group_layouts: &[&ubo_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Test Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
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
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let combined_buffer = create_combined_buffer(&device);

        Self {
            pipeline,
            combined_buffer,
            ubo_buffer,
            ubo_bind_group,
            start_time: std::time::Instant::now(),
        }
    }

    fn update_uniform_buffer(&mut self, device: &Device, window_width: u32, window_height: u32) {
        let time = self.start_time.elapsed().as_secs_f32();

        // 모델 행렬: Z축 회전
        let model = glam::Mat4::from_rotation_z(time.to_radians() * 90.0);

        // 뷰 행렬: 카메라 위치 설정
        let eye = glam::Vec3::new(2.0, 2.0, 2.0);
        let center = glam::Vec3::ZERO;
        let up = glam::Vec3::Z;
        let view = glam::Mat4::look_at_rh(eye, center, up);

        // 프로젝션 행렬
        let aspect_ratio = window_width as f32 / window_height as f32;
        let proj = glam::Mat4::perspective_rh(45.0_f32.to_radians(), aspect_ratio, 0.1, 10.0);

        let ubo = UniformBufferObject { model, view, proj };

        let ubo_bytes = unsafe {
            std::slice::from_raw_parts(
                &ubo as *const UniformBufferObject as *const u8,
                std::mem::size_of::<UniformBufferObject>(),
            )
        };

        // 메모리에 데이터 복사
        device.queue.write_buffer(&self.ubo_buffer, 0, ubo_bytes);
    }

    pub fn record_commands(
        &mut self,
        device: &Device,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window_width: u32,
        window_height: u32,
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

        render_pass.set_vertex_buffer(
            0,
            self.combined_buffer
                .buffer
                .slice(self.combined_buffer.vertex_offset..),
        );

        render_pass.set_index_buffer(
            self.combined_buffer
                .buffer
                .slice(self.combined_buffer.index_offset..),
            wgpu::IndexFormat::Uint16,
        );

        self.update_uniform_buffer(device, window_width, window_height);
        render_pass.set_bind_group(0, &self.ubo_bind_group, &[]);

        render_pass.draw_indexed(0..self.combined_buffer.index_count, 0, 0..1);
    }
}
