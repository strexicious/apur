use crate::mesh::Mesh;

pub struct Object<M> {
    mat_name: String,
    material: M,
    mesh: Mesh,
}
