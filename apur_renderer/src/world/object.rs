use std::rc::Rc;

use crate::mesh::Mesh;
use super::material::Material;

pub struct Object<M: Material> {
    material: Rc<M>,
    mesh: Mesh,
}

impl<M: Material> Object<M> {
    pub fn new(mesh: Mesh, material: Rc<M>) -> Self {
        Self { mesh, material }
    }
    
    pub fn get_mesh(&self) -> &Mesh {
        &self.mesh
    }

    pub fn get_material(&self) -> &M {
        &self.material
    }
}
