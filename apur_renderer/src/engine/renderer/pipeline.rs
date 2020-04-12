use super::super::model::{Mesh};
use super::super::material::{MaterialManager};

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    global_bg: wgpu::BindGroup,
    meshes: Vec<Mesh>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_source: (&[u8], &[u8]),
        global_layout: &wgpu::BindGroupLayoutDescriptor,
        global_bindings: &[wgpu::Binding],
        mat_layout: &wgpu::BindGroupLayout,
        vertex_state: &wgpu::VertexStateDescriptor,
    ) -> Self {
        let global_layout = device.create_bind_group_layout(global_layout);
    
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&global_layout, &mat_layout],
        });
    
        let vmodule = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&shader_source.0[..])).expect("failed to read vertex shader spir-v"));
    
        let fmodule = device.create_shader_module(&wgpu::read_spirv(
            std::io::Cursor::new(&shader_source.1[..])).expect("failed to read fragment shader spir-v"));
    
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vmodule,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fmodule,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::default(),
                stencil_back: wgpu::StencilStateFaceDescriptor::default(),
                stencil_read_mask: !0,
                stencil_write_mask: !0,
            }),
            vertex_state: vertex_state.clone(),
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let global_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &global_layout,
            bindings: global_bindings,
            label: None,
        });

        Self {
            pipeline,
            global_bg,
            meshes: vec![],
        }
    }

    pub fn add_mesh(&mut self, mesh: Mesh) {
        self.meshes.push(mesh);
    }

    pub fn draw_meshes<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>, mat_man: &'a MaterialManager) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.global_bg, &[]);

        for m in self.meshes.iter() {
            let vbuf = m.get_vertex_buffer();
            let ibuf = m.get_indices_buffer();
            rpass.set_vertex_buffer(0, vbuf.get_buffer(), 0, vbuf.get_size_bytes() as u64);
            rpass.set_index_buffer(ibuf.get_buffer(), 0, ibuf.get_size_bytes() as u64);

            let mat_bg = mat_man.get_material(m.get_mat_name()).unwrap().get_mat_bg();
            rpass.set_bind_group(1, mat_bg, &[]);
            rpass.draw_indexed(0..ibuf.get_size() as u32, 0, 0..1);
        }
    }
}
