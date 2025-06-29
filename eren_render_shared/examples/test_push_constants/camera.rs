use glam::Mat4;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct CameraUBO {
    pub view: Mat4,
    pub proj: Mat4,
}
