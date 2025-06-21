use crate::mesh::MeshGroup;

pub struct MaterialGroup {
    material_id: u32,
    mesh_groups: Vec<MeshGroup>,
}
