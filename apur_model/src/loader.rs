use apur_renderer::buffer::ManagedBuffer;

pub fn load_raw_model_positions(device: &wgpu::Device, obj_filename: &str) -> ManagedBuffer {
    let (models, _) = tobj::load_obj(format!("res/models/{}.obj", obj_filename), true)
        .expect("Failed to load the model");

    let vertices = models
        .iter()
        .flat_map(|m| {
            m.mesh
                .indices
                .iter()
                .flat_map(move |i| m.mesh.positions.iter().skip(*i as usize * 3).take(3).copied())
        })
        .collect::<Vec<f32>>();

    ManagedBuffer::from_data(device, wgpu::BufferUsage::VERTEX, &vertices)
}
