#[repr(C)]
#[derive(Copy, Clone)]
pub struct ComputePushConstants {
    pub time: f32,
    pub aspect_ratio: f32,
    pub pre_transform: u32,
    pub _padding: u32,
}
