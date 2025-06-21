use std::cmp::Ordering;

use glam::{Mat4, Vec3, Vec4};

pub struct TransparentInstance {
    pub material_id: u32,
    pub mesh_id: u32,
    pub transform: Mat4,
    pub color: Vec4,
    pub world_position: Vec3, // 중심 좌표
}

pub struct TransparentPhase {
    pub instances: Vec<TransparentInstance>,
}

impl TransparentPhase {
    pub fn sort(&mut self, camera_pos: Vec3) {
        self.instances.sort_by(|a, b| {
            let a_dist2 = (a.world_position - camera_pos).length_squared();
            let b_dist2 = (b.world_position - camera_pos).length_squared();

            match b_dist2.partial_cmp(&a_dist2).unwrap_or(Ordering::Equal) {
                Ordering::Equal => match a.material_id.cmp(&b.material_id) {
                    Ordering::Equal => a.mesh_id.cmp(&b.mesh_id),
                    other => other,
                },
                other => other,
            }
        });
    }
}
