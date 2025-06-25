use glam::{Mat4, Quat, Vec3};

pub struct CameraUBO {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,

    pub fov_y_radians: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub ubo: CameraUBO,
}

impl Camera {
    pub fn update_ubo(&mut self) {
        let forward = self.rotation * Vec3::NEG_Z;
        let up = self.rotation * Vec3::Y;

        let view = Mat4::look_to_rh(self.position, forward, up);
        let proj = Mat4::perspective_rh(self.fov_y_radians, self.aspect, self.near, self.far);

        self.ubo.view_proj = (proj * view).to_cols_array_2d();
        self.ubo.camera_pos = self.position.to_array();
    }
}
