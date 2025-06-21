use glam::{Affine2, Vec4};

pub struct TransparentInstance {
    pub material_id: u32,
    pub mesh_id: u32,
    pub transform: Affine2,
    pub color: Vec4,
    pub z: f32, // 깊이 정렬용
}

pub struct TransparentPhase {
    pub instances: Vec<TransparentInstance>,
}

impl TransparentPhase {
    pub fn sort(&mut self) {
        self.instances.sort_by(|a, b| {
            // back-to-front: 높은 z가 뒤쪽
            b.z.partial_cmp(&a.z)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.material_id.cmp(&b.material_id))
                .then_with(|| a.mesh_id.cmp(&b.mesh_id))
        });
    }
}
