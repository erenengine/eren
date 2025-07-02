use chrono::Utc;
use eren_render_shared::{adapter::Adapter, device::Device, surface::Surface};
use glam::{Mat4, Vec3, vec3};

use crate::test_shadow::{debug_quad_pass::DebugQuadPass, mesh::MeshBuffer, ubo::ShadowUBO};

use super::shadow_pass::ShadowPass;

const DEBUG_QUAD_PASS_ENABLED: bool = true;

pub struct TestRenderer {
    shadow_pass: ShadowPass,
    debug_quad_pass: DebugQuadPass,

    start_time: chrono::DateTime<chrono::Utc>,
}

impl TestRenderer {
    pub fn new(adapter: &Adapter, device: &Device, window_width: u32, window_height: u32) -> Self {
        let shadow_pass =
            ShadowPass::new(device, adapter.depth_format, window_width, window_height);
        let debug_quad_pass = DebugQuadPass::new(device, &shadow_pass.depth_texture_view);
        Self {
            shadow_pass,
            debug_quad_pass,

            start_time: Utc::now(),
        }
    }

    pub fn render(
        &mut self,
        surface: &Surface,
        device: &Device,
        mesh_buffers: &[MeshBuffer],
    ) -> Result<(), wgpu::SurfaceError> {
        let output = surface.get_current_texture()?;
        let view = output
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
        let light_x = radius * (-speed * 2.0 * time).cos();
        let light_z = -radius * (-speed * 2.0 * time).sin();
        let light_pos = vec3(light_x, height, light_z);

        // Shadow-Pass용 뷰·프로젝션 행렬
        let light_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
        let light_view = Mat4::look_at_rh(light_pos, Vec3::ZERO, Vec3::Y);
        let light_view_proj = light_proj * light_view;

        // Shadow UBO 갱신
        self.shadow_pass
            .update_shadow_ubo(&device.queue, ShadowUBO { light_view_proj });

        self.shadow_pass.record_commands(&mut encoder, mesh_buffers);
        self.debug_quad_pass.record_commands(&mut encoder, &view);

        device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
