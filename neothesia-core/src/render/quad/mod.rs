mod instance_data;
pub use instance_data::QuadInstance;

use wgpu_jumpstart::{
    wgpu, Gpu, Instances, RenderPipelineBuilder, Shape, TransformUniform, Uniform,
};

pub struct QuadPipeline {
    render_pipeline: wgpu::RenderPipeline,
    quad: Shape,
    instances: Instances<QuadInstance>,
}

impl<'a> QuadPipeline {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RectanglePipeline::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "./shader.wgsl"
                ))),
            });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&transform_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let ri_attrs = QuadInstance::attributes();

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = wgpu::RenderPipelineDescriptor::builder(
            &render_pipeline_layout,
            wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Shape::layout(), QuadInstance::layout(&ri_attrs)],
            },
        )
        .fragment("fs_main", &shader, &[Some(target)])
        .create_render_pipeline(&gpu.device);

        let quad = Shape::new_quad(&gpu.device);
        let instances = Instances::new(&gpu.device, 100_000);

        Self {
            render_pipeline,
            quad,
            instances,
        }
    }

    pub fn render(
        &'a self,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..self.instances.len());
    }

    pub fn clear(&mut self) {
        self.instances.data.clear();
    }

    pub fn instances(&mut self) -> &mut Vec<QuadInstance> {
        &mut self.instances.data
    }

    pub fn prepare(&self, queue: &wgpu::Queue) {
        self.instances.update(queue);
    }

    pub fn update_instance_buffer(&mut self, queue: &wgpu::Queue, instances: Vec<QuadInstance>) {
        self.instances.data = instances;
        self.instances.update(queue);
    }

    pub fn with_instances_mut<F: FnOnce(&mut Vec<QuadInstance>)>(
        &mut self,
        queue: &wgpu::Queue,
        cb: F,
    ) {
        cb(&mut self.instances.data);
        self.instances.update(queue);
    }
}
