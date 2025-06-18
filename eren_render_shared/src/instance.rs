use std::sync::Arc;

use winit::window::Window;

pub struct Instance {
    pub window: Arc<Window>,
}

impl Instance {
    pub fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}
