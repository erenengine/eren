use glam::{Mat4, Vec2, Vec3};

pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,     // in radians
    pub screen_size: Vec2, // in pixels or world units
    pub view_proj_matrix: Mat4,
}

impl Camera {
    pub fn new(position: Vec2, zoom: f32, rotation: f32, screen_size: Vec2) -> Self {
        let mut cam = Self {
            position,
            zoom,
            rotation,
            screen_size,
            view_proj_matrix: Mat4::IDENTITY,
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

        self.view_proj_matrix = proj * view;
    }
}
