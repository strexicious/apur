pub enum Geometry {
    RigidMesh(Vec<f32>), // positions
    // water
    // cloth
    // etc
}

pub enum FixedMapped<T> {
    Fixed(T),
    Mapped(String), // texture name
}

pub struct Material {
    color: FixedMapped<(f32, f32, f32, f32)>,
    specular: Option<FixedMapped<f32>>,
    roughness: Option<FixedMapped<f32>>,
    bump: Option<String>,
    displacement: Option<String>,
    light_map: Option<String>,
}

pub struct Renderable {
    geometry: Geometry,
    material: Material,
}
