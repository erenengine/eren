pub mod camera;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod transparent;
pub mod viewport;

pub struct RenderView {
    viewport: viewport::Viewport,
    camera: camera::Camera,
    opaque_material_groups: Vec<material::MaterialGroup>,
    transparent_phase: transparent::TransparentPhase,
}
