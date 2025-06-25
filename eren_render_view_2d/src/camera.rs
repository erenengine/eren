use glam::{Mat4, Vec2, Vec3};

pub struct CameraUBO {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 2],
    pub _padding: [f32; 2],
}

pub struct Camera {
    position: Vec2,
    zoom: f32,
    rotation: f32,     // in radians
    screen_size: Vec2, // in pixels or world units

    pub ubo: CameraUBO,
}

impl Camera {
    pub fn new(position: Vec2, zoom: f32, rotation: f32, screen_size: Vec2) -> Self {
        let mut cam = Self {
            position,
            zoom,
            rotation,
            screen_size,
            ubo: CameraUBO {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                camera_pos: position.to_array(),
                _padding: [0.0; 2],
            },
        };
        cam.update();
        cam
    }

    pub fn update(&mut self) {
        let half_w = self.screen_size.x / 2.0 / self.zoom;
        let half_h = self.screen_size.y / 2.0 / self.zoom;

        let proj = Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, -1.0, 1.0);

        let view = Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0))
            * Mat4::from_rotation_z(-self.rotation);

        self.ubo.view_proj = (proj * view).to_cols_array_2d();
        self.ubo.camera_pos = self.position.to_array();
    }
}
