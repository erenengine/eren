pub mod camera;
pub mod directional_light;
pub mod instance;
pub mod material;
pub mod mesh;
pub mod transparent;
pub mod viewport;

pub struct RenderView {
    viewport: viewport::Viewport,
    camera: camera::Camera,
    lights: Vec<directional_light::DirectionalLight>,
    opaque_material_groups: Vec<material::MaterialGroup>,
    transparent_phase: transparent::TransparentPhase,
}
