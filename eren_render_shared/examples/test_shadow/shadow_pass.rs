use eren_render_shared::device::Device;

use crate::test_shadow::{mesh::MeshBuffer, ubo::ShadowUBO, vertex::VERTEX_DESC};

const SHADER_STR: &str = include_str!("./shaders/shadow.wgsl");

const CLEAR_COLOR: f32 = 1.0;

pub struct ShadowPass {
    pub depth_texture_view: wgpu::TextureView,

    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
}

impl ShadowPass {
    pub fn new(
        device: &Device,
        depth_format: wgpu::TextureFormat,
        window_width: u32,
        window_height: u32,
    ) -> Self {
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shadow Shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_STR.into()),
        });

        // 깊이 텍스처 생성
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Shadow Depth Texture"),
            size: wgpu::Extent3d {
                width: window_width,
                height: window_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shadow UBO"),
            size: std::mem::size_of::<ShadowUBO>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow Bind Group Layout"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow Bind Group"),
            layout: &shadow_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shadow Pipeline Layout"),
            bind_group_layouts: &[&shadow_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[VERTEX_DESC],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: None, // depth only
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                front_face: wgpu::FrontFace::Ccw,
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        Self {
            depth_texture_view,

            uniform_buffer,
            bind_group,
            pipeline,
        }
    }

    pub fn update_shadow_ubo(&mut self, queue: &wgpu::Queue, shadow_ubo: ShadowUBO) {
        let ubo_bytes = unsafe {
            std::slice::from_raw_parts(
                &shadow_ubo as *const ShadowUBO as *const u8,
                std::mem::size_of::<ShadowUBO>(),
            )
        };

        // 메모리에 데이터 복사
        queue.write_buffer(&self.uniform_buffer, 0, ubo_bytes);
    }

    pub fn record_commands(&mut self, encoder: &mut wgpu::CommandEncoder, meshes: &[MeshBuffer]) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Shadow Render Pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        for mesh in meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(0..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(0..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
