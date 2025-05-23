use eren_core::Context;

pub struct Renderer {}

impl Renderer {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        Context::new(width, height, title, |path| {});
        Self {}
    }
}
