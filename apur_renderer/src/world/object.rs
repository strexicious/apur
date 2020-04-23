use crate::mesh::Mesh;
use super::material::Material;

pub struct Object<M: Material> {
    material: M,
    mesh: Mesh,
}

impl<M: Material> Object<M> {
    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn get_material(&self) -> &M {
        &self.material
    }
}
