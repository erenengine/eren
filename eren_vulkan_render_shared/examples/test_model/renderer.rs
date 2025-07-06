use std::sync::Arc;

use ash::vk;
use eren_vulkan_render_shared::{
    command::CommandPool,
    device::{
        CommandBufferBeginError, CommandBufferEndError, CommandBufferResetError, Device,
        ImageViewCreationError, ImageWithMemoryCreationError, ResetFencesError,
        SubmitGraphicsCommandsError, WaitForFencesError,
    },
    frame::{FrameManager, FrameManagerInitializationError},
    physical_device::PhysicalDevice,
    swapchain::{Swapchain, SwapchainAcquireError, SwapchainPresentError},
};
use glam::{Mat4, Vec3, vec3};
use thiserror::Error;

use super::{
    debug_quad_pass::DebugQuadPass,
    main_pass::{TestMainPass, TestMainPassInitializationError},
    mesh::MeshBuffer,
    shadow_pass::{TestShadowPass, TestShadowPassInitializationError},
    ubo::{LightUBO, MainUBO, ShadowUBO},
};

use super::debug_quad_pass::DebugQuadPassInitializationError;

const DEBUG_QUAD_PASS_ENABLED: bool = false;

pub struct TestRenderer {
    device: Arc<Device>,
    swapchain: Arc<Swapchain>,
    frame_mgr: FrameManager,
    shadow_pass: TestShadowPass,
    debug_quad_pass: DebugQuadPass,
    main_pass: TestMainPass,

    start_time: std::time::Instant,
}

#[derive(Debug, Error)]
pub enum TestRendererInitializationError {
    #[error("Failed to create frame manager: {0}")]
    CreateFrameManager(#[from] FrameManagerInitializationError),

    #[error("Failed to create image with memory: {0}")]
    CreateImageWithMemory(#[from] ImageWithMemoryCreationError),

    #[error("Failed to create image view: {0}")]
    CreateImageView(#[from] ImageViewCreationError),

    #[error("Failed to create shadow pass: {0}")]
    CreateShadowPass(#[from] TestShadowPassInitializationError),

    #[error("Failed to create render pass: {0}")]
    CreateDebugQuadPass(#[from] DebugQuadPassInitializationError),

    #[error("Failed to create main pass: {0}")]
    CreateMainPass(#[from] TestMainPassInitializationError),
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to wait for fences: {0}")]
    WaitForFences(#[from] WaitForFencesError),

    #[error("Failed to reset fences: {0}")]
    ResetFences(#[from] ResetFencesError),

    #[error("Failed to reset command buffer: {0}")]
    ResetCommandBuffer(#[from] CommandBufferResetError),

    #[error("Failed to begin command buffer: {0}")]
    BeginCommandBuffer(#[from] CommandBufferBeginError),

    #[error("Failed to end command buffer: {0}")]
    EndCommandBuffer(#[from] CommandBufferEndError),

    #[error("Failed to acquire next image: {0}")]
    AcquireNextImage(#[from] SwapchainAcquireError),

    #[error("Failed to submit graphics commands: {0}")]
    SubmitGraphicsCommands(#[from] SubmitGraphicsCommandsError),

    #[error("Failed to present: {0}")]
    Present(#[from] SwapchainPresentError),
}

/*pub fn transition_image_layout(
    device: &Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    aspect_mask: vk::ImageAspectFlags,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::SHADER_READ)
        .image(image)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    device.pipeline_barrier(
        cmd,
        vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
        vk::PipelineStageFlags::FRAGMENT_SHADER,
        vk::DependencyFlags::empty(),
        &[],
        &[],
        &[barrier],
    );
}*/

impl TestRenderer {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        swapchain: Arc<Swapchain>,
        command_pool: &CommandPool,
        render_area: vk::Rect2D,
    ) -> Result<Self, TestRendererInitializationError> {
        let frame_mgr = FrameManager::new(device.clone(), command_pool, swapchain.image_len)?;

        let shadow_pass = TestShadowPass::new(physical_device, device.clone(), render_area)?;

        let debug_quad_pass = DebugQuadPass::new(
            device.clone(),
            &swapchain,
            render_area,
            shadow_pass.depth_attachment.view,
        )?;

        let main_pass = TestMainPass::new(
            physical_device,
            device.clone(),
            render_area,
            &swapchain,
            shadow_pass.depth_attachment.view,
        )?;

        Ok(Self {
            device,
            swapchain,
            frame_mgr,
            shadow_pass,
            debug_quad_pass,
            main_pass,

            start_time: std::time::Instant::now(),
        })
    }

    pub fn render(&mut self, mesh_buffers: &[MeshBuffer]) -> Result<bool, RenderError> {
        let (frame, frame_idx) = self.frame_mgr.next_frame();
        let (image_available, in_flight, cmd_buffer) =
            { (frame.image_available, frame.in_flight, frame.cmd_buffer) };

        // 이전 프레임 GPU 작업 완료 대기
        self.device.wait_for_fence(in_flight)?;
        self.device.reset_fence(in_flight)?;

        let (swapchain_image_idx, is_suboptimal) = self.swapchain.acquire_next_image(
            image_available, // wait
        )?;

        if is_suboptimal {
            log::debug!("Swapchain is suboptimal when acquire next image");
            return Ok(true);
        }

        // 이미지 전용 세마포어 가져오기
        let img = self.frame_mgr.swapchain_image(swapchain_image_idx as usize);

        self.device.reset_command_buffer(cmd_buffer)?;
        self.device.begin_command_buffer(cmd_buffer)?;

        let time = self.start_time.elapsed().as_secs_f32();
        let speed = 0.2; // 회전 속도(라디언/초) – 느리게 돌리려면 더 작게
        let radius = 8.0; // 원 궤도의 반지름
        let height = 6.0; // 카메라 고도(Y 좌표)

        // 빛 위치를 궤도 위에서 계산 (카메라는 +speed, 빛은 -speed * 2)
        let light_x = radius * (-speed * 2.0 * time).cos();
        let light_z = -radius * (-speed * 2.0 * time).sin();
        let light_pos = vec3(light_x, height, light_z);

        // Shadow-Pass용 뷰·프로젝션 행렬
        let mut light_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, -10.0, 20.0);
        light_proj.y_axis.y *= -1.0; // Vulkan Y-flip
        let light_view = Mat4::look_at_rh(light_pos, Vec3::ZERO, Vec3::Y);
        let light_view_proj = light_proj * light_view;

        // Shadow UBO 갱신
        self.shadow_pass
            .update_shadow_ubo(ShadowUBO { light_view_proj });

        self.shadow_pass.record_commands(cmd_buffer, &mesh_buffers);

        /*transition_image_layout(
            &self.device,
            cmd_buffer,
            self.shadow_pass.depth_attachment.image,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::ImageAspectFlags::DEPTH,
        );*/

        if DEBUG_QUAD_PASS_ENABLED {
            self.debug_quad_pass
                .record_commands(cmd_buffer, swapchain_image_idx as usize);
        } else {
            let cam_x = radius * (-speed * time).cos();
            let cam_z = radius * (-speed * time).sin();
            let camera_pos = vec3(cam_x, height, cam_z);

            let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
            let mut proj = Mat4::perspective_rh(
                15.0f32.to_radians(),
                self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32,
                0.1,
                100.0,
            );

            // glam은 Y-up, Vulkan은 Y-down. Y축을 뒤집어 보정합니다.
            proj.y_axis.y *= -1.0;

            self.main_pass.update_main_ubo(
                frame_idx,
                MainUBO {
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
            self.main_pass.update_light_ubo(frame_idx, light_ubo);

            self.main_pass.record_commands(
                cmd_buffer,
                swapchain_image_idx as usize,
                frame_idx,
                &mesh_buffers,
            );
        }

        self.device.end_command_buffer(cmd_buffer)?;

        self.device.submit_graphics_commands(
            cmd_buffer,
            image_available,
            img.render_finished,
            in_flight,
        )?;

        let is_suboptimal =
            self.device
                .present(&self.swapchain, swapchain_image_idx, img.render_finished)?;

        if is_suboptimal {
            log::debug!("Swapchain is suboptimal when present");
        }

        Ok(is_suboptimal)
    }
}
