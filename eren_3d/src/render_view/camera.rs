use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,

    pub fov_y_radians: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,

    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub view_proj_matrix: Mat4,
}

impl Camera {
    pub fn new_perspective(
        position: Vec3,
        rotation: Quat,
        fov_y_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut cam = Self {
            position,
            rotation,
            fov_y_radians,
            aspect,
            near,
            far,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            view_proj_matrix: Mat4::IDENTITY,
        };
        cam.update_matrices();
        cam
    }

    pub fn update_matrices(&mut self) {
        // 뷰 행렬: 카메라의 위치와 회전 기준
        let forward = self.rotation * Vec3::NEG_Z;
        let up = self.rotation * Vec3::Y;
        self.view_matrix = Mat4::look_to_rh(self.position, forward, up);

        // 원근 투영 행렬
        self.projection_matrix =
            Mat4::perspective_rh(self.fov_y_radians, self.aspect, self.near, self.far);

        self.view_proj_matrix = self.projection_matrix * self.view_matrix;
    }
}
