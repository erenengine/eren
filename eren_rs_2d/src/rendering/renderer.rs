use super::{context::Context, create_window::create_window};

pub struct Renderer {}

impl Renderer {
    pub async fn new(width: u32, height: u32, title: &str) -> Self {
        let window = create_window(width, height, title).await;
        let context = Context::new(window).await;
        Self {}
    }
}
