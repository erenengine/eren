use chrono::Utc;
use eren_render_shared::{adapter::Adapter, device::Device, surface::Surface};
use glam::{Mat4, Vec3, vec3};

use super::{
    //debug_quad_pass::DebugQuadPass,
    main_pass::MainPass,
    mesh::MeshBuffer,
    ubo::{LightUBO, MainUBO, ShadowUBO},
};

use super::shadow_pass::ShadowPass;

const DEBUG_QUAD_PASS_ENABLED: bool = false;

pub struct TestRenderer {
    shadow_pass: ShadowPass,
    //debug_quad_pass: DebugQuadPass,
    main_pass: MainPass,

    start_time: chrono::DateTime<chrono::Utc>,
}

impl TestRenderer {
    pub fn new(adapter: &Adapter, device: &Device, window_width: u32, window_height: u32) -> Self {
        let shadow_pass =
            ShadowPass::new(device, adapter.depth_format, window_width, window_height);
        //let debug_quad_pass = DebugQuadPass::new(device, &shadow_pass.shadow_texture_view);
        let main_pass = MainPass::new(
            device,
            adapter.preferred_surface_format,
            adapter.depth_format,
            &shadow_pass.shadow_texture_view,
            window_width,
            window_height,
        );
        Self {
            shadow_pass,
            //debug_quad_pass,
            main_pass,

            start_time: Utc::now(),
        }
    }

    pub fn resize(
        &mut self,
        device: &Device,
        depth_format: wgpu::TextureFormat,
        window_width: u32,
        window_height: u32,
    ) {
        self.shadow_pass
            .resize_shadow_texture(device, depth_format, window_width, window_height);

        /*self.debug_quad_pass
        .rebind_shadow_texture(device, &self.shadow_pass.shadow_texture_view);*/

        self.main_pass.resize_depth_texture(
            device,
            &self.shadow_pass.shadow_texture_view,
            depth_format,
            window_width,
            window_height,
        );
    }

    pub fn render(
        &mut self,
        surface: &Surface,
        device: &Device,
        mesh_buffers: &[MeshBuffer],
        window_width: u32,
        window_height: u32,
    ) -> Result<(), wgpu::SurfaceError> {
        let output: wgpu::SurfaceTexture = surface.get_current_texture()?;
        let surface_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Test Render Encoder"),
        });

        let time = self
            .start_time
            .signed_duration_since(Utc::now())
            .num_milliseconds() as f32
            / 1000.0;

        let speed = 0.2; // 회전 속도(라디언/초) – 느리게 돌리려면 더 작게
        let radius = 8.0; // 원 궤도의 반지름
        let height = 6.0; // 카메라 고도(Y 좌표)

        // 빛 위치를 궤도 위에서 계산 (카메라는 +speed, 빛은 -speed * 2)
        let light_x = radius * (speed * 2.0 * time).cos();
        let light_z = -radius * (speed * 2.0 * time).sin();
        let light_pos = vec3(light_x, height, light_z);

        // Shadow-Pass용 뷰·프로젝션 행렬
        let light_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
        let light_view = Mat4::look_at_rh(light_pos, Vec3::ZERO, Vec3::Y);
        let light_view_proj = light_proj * light_view;

        // Shadow UBO 갱신
        self.shadow_pass
            .update_shadow_ubo(&device.queue, ShadowUBO { light_view_proj });

        self.shadow_pass.record_commands(&mut encoder, mesh_buffers);

        if DEBUG_QUAD_PASS_ENABLED {
            /*self.debug_quad_pass
            .record_commands(&mut encoder, &surface_view);*/
        } else {
            let cam_x = radius * (-speed * time).cos();
            let cam_z = radius * (-speed * time).sin();
            //let cam_x = radius;
            //let cam_z = radius;
            let camera_pos = vec3(cam_x, height, cam_z);

            let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
            let proj = Mat4::perspective_rh(
                45.0f32.to_radians(),
                window_width as f32 / window_height as f32,
                0.1,
                100.0,
            );

            self.main_pass.update_main_ubo(
                &device.queue,
                &MainUBO {
                    model: Mat4::IDENTITY,
                    view,
                    proj,
                    light_view_proj,
                },
            );

            // 조명 정보 설정
            let light_dir = (Vec3::ZERO - light_pos).normalize();
            let light_ubo = LightUBO {
                direction: light_dir,
                _pad1: 0.0,
                color: vec3(1.0, 1.0, 1.0),
                _pad2: 0.0,
            };
            self.main_pass.update_light_ubo(&device.queue, &light_ubo);

            self.main_pass
                .record_commands(&mut encoder, &surface_view, &mesh_buffers);
        }

        device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
