use glam::{Mat4, Vec3};

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct ShadowUBO {
    pub light_view_proj: Mat4,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MainUBO {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
    pub light_view_proj: Mat4,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUBO {
    pub direction: Vec3,
    pub _pad1: f32,
    pub color: Vec3,
    pub _pad2: f32,
}
