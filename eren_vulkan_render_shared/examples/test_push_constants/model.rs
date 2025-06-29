use glam::Mat4;

#[repr(C, align(16))]
#[derive(Copy, Clone)]
pub struct PushModel {
    pub model: Mat4,
}
