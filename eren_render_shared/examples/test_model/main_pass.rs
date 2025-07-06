use super::{
    mesh::MeshBuffer,
    ubo::{LightUBO, MainUBO},
    vertex::VERTEX_DESC,
};
use eren_render_shared::device::Device;
use image::{GenericImageView, ImageError};

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1921,
    g: 0.302,
    b: 0.4745,
    a: 1.0,
};

const CLEAR_DEPTH: f32 = 1.0;

pub struct MainPass {
    pipeline: wgpu::RenderPipeline,

    main_ubo_buffer: wgpu::Buffer,
    light_ubo_buffer: wgpu::Buffer,
    bind_group_main: wgpu::BindGroup, // set=0

    shadow_sampler: wgpu::Sampler,
    texture_image_view: wgpu::TextureView,
    texture_sampler: wgpu::Sampler,

    bind_group_layout_shadow_and_texture: wgpu::BindGroupLayout,
    bind_group_shadow_and_texture: wgpu::BindGroup, // set=1

    scene_depth_view: wgpu::TextureView,
}

impl MainPass {
    pub fn new(
        device: &Device,
        surface_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
        shadow_texture_view: &wgpu::TextureView,
        window_width: u32,
        window_height: u32,
        image_bytes: &[u8],
    ) -> Result<Self, ImageError> {
        let diffuse_image = image::load_from_memory(image_bytes)?;
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Test Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        device.queue.write_texture(
            // wgpu에게 픽셀 데이터를 어디에 복사할지 알려줍니다.
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // 실제 픽셀 데이터
            &diffuse_rgba,
            // 텍스처의 레이아웃
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let texture_image_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Main Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/main.wgsl").into()),
        });

        // --- UBO Layout (set=0) ---
        let bind_group_layout_main =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Main UBO Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0, // MainUBO
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1, // LightUBO
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // --- Shadow and Texture Layout (set=1) ---
        let bind_group_layout_shadow_and_texture =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shadow and Texture Sampler Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // --- Shadow Sampler ---
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            //border_color: Some(wgpu::SamplerBorderColor::OpaqueWhite), // WebGL에서는 지원하지 않음
            compare: Some(wgpu::CompareFunction::Less),
            ..Default::default()
        });

        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let bind_group_shadow_and_texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow and Texture Bind Group"),
            layout: &bind_group_layout_shadow_and_texture,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(shadow_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_image_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        // --- Pipeline Layout ---
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout_main,
                &bind_group_layout_shadow_and_texture,
            ],
            push_constant_ranges: &[],
        });

        // --- Buffers & Bind Groups for UBOs ---
        let main_ubo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Main UBO"),
            size: std::mem::size_of::<MainUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let light_ubo_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Light UBO"),
            size: std::mem::size_of::<LightUBO>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_main = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: &bind_group_layout_main,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: main_ubo_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_ubo_buffer.as_entire_binding(),
                },
            ],
        });

        // --- Render Pipeline ---
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VERTEX_DESC],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
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

        Ok(Self {
            pipeline,

            main_ubo_buffer,
            light_ubo_buffer,
            bind_group_main,

            shadow_sampler,
            texture_image_view,
            texture_sampler,

            bind_group_layout_shadow_and_texture,
            bind_group_shadow_and_texture,

            scene_depth_view: Self::create_depth_texture_view(
                device,
                depth_format,
                window_width,
                window_height,
            ),
        })
    }

    fn create_depth_texture_view(
        device: &Device,
        depth_format: wgpu::TextureFormat,
        window_width: u32,
        window_height: u32,
    ) -> wgpu::TextureView {
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize_depth_texture(
        &mut self,
        device: &Device,
        shadow_texture_view: &wgpu::TextureView,
        depth_format: wgpu::TextureFormat,
        window_width: u32,
        window_height: u32,
    ) {
        self.bind_group_shadow_and_texture = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Shadow and Texture Bind Group"),
            layout: &self.bind_group_layout_shadow_and_texture,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(shadow_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.shadow_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.texture_image_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.texture_sampler),
                },
            ],
        });

        self.scene_depth_view =
            Self::create_depth_texture_view(device, depth_format, window_width, window_height);
    }

    pub fn update_main_ubo(&self, queue: &wgpu::Queue, ubo: &MainUBO) {
        queue.write_buffer(&self.main_ubo_buffer, 0, bytemuck::bytes_of(ubo));
    }

    pub fn update_light_ubo(&self, queue: &wgpu::Queue, ubo: &LightUBO) {
        queue.write_buffer(&self.light_ubo_buffer, 0, bytemuck::bytes_of(ubo));
    }

    pub fn record_commands(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        meshes: &[MeshBuffer],
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.scene_depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(CLEAR_DEPTH),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group_main, &[]);
        pass.set_bind_group(1, &self.bind_group_shadow_and_texture, &[]);

        for mesh in meshes {
            pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}
