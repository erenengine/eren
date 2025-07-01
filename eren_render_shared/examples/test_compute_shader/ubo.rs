#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UniformBufferObject {
    pub time: f32,
    pub aspect_ratio: f32,
    pub _padding: [f32; 2],
}
