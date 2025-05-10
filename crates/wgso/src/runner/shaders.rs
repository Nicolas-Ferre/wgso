use crate::directives::Directive;
use crate::program::module::Module;
use crate::runner::gpu;
use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, CompareFunction,
    ComputePipeline, ComputePipelineDescriptor, DepthBiasState, DepthStencilState, Device,
    FrontFace, MultisampleState, PipelineCompilationOptions, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderStages, StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

#[derive(Debug)]
pub(crate) struct ComputeShaderResources {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) layout: Option<BindGroupLayout>,
    pub(crate) directive: Directive,
}

impl ComputeShaderResources {
    pub(crate) fn new(directive: &Directive, module: &Module, device: &Device) -> Self {
        let layout = (module.binding_count() > 0)
            .then(|| create_bind_group_layout(directive, module, device, ShaderStages::COMPUTE));
        let pipeline = Self::create_pipeline(module, directive, device, layout.as_ref());
        Self {
            pipeline,
            layout,
            directive: directive.clone(),
        }
    }

    fn create_pipeline(
        module: &Module,
        directive: &Directive,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> ComputePipeline {
        let directive_code = directive.code();
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&directive_code),
            source: wgpu::ShaderSource::Wgsl(module.code.as_str().into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&directive_code),
            layout: Some(&gpu::pipeline_layout(device, layout, &directive_code)),
            module: &module,
            entry_point: None,
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}

#[derive(Debug)]
pub(crate) struct RenderShaderResources {
    pub(crate) pipeline: RenderPipeline,
    pub(crate) layout: Option<BindGroupLayout>,
}

impl RenderShaderResources {
    pub(crate) fn new(
        directive: &Directive,
        module: &Module,
        texture_format: TextureFormat,
        device: &Device,
    ) -> Self {
        let layout = (module.binding_count() > 0).then(|| {
            create_bind_group_layout(directive, module, device, ShaderStages::VERTEX_FRAGMENT)
        });
        let pipeline =
            Self::create_pipeline(module, directive, texture_format, device, layout.as_ref());
        Self { pipeline, layout }
    }

    fn create_pipeline(
        directive_module: &Module,
        directive: &Directive,
        texture_format: TextureFormat,
        device: &Device,
        layout: Option<&BindGroupLayout>,
    ) -> RenderPipeline {
        let directive_code = directive.code();
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&directive_code),
            source: wgpu::ShaderSource::Wgsl(directive_module.code.as_str().into()),
        });
        let vertex_type = &directive_module
            .type_(&directive.vertex_type().slice)
            .expect("internal error: vertex type should be validated");
        let instance_type = &directive_module
            .type_(&directive.instance_type().slice)
            .expect("internal error: instance type should be validated");
        let first_instance_location = vertex_type.fields.len();
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&directive_code),
            layout: Some(&gpu::pipeline_layout(device, layout, &directive_code)),
            vertex: VertexState {
                module: &module,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[
                    VertexBufferLayout {
                        array_stride: vertex_type.size.into(),
                        step_mode: VertexStepMode::Vertex,
                        attributes: &vertex_type
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(location, field)| {
                                Self::attribute(&field.type_.label, field.type_.offset, location)
                            })
                            .collect::<Vec<_>>(),
                    },
                    VertexBufferLayout {
                        array_stride: instance_type.size.into(),
                        step_mode: VertexStepMode::Instance,
                        attributes: &instance_type
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(location, field)| {
                                Self::attribute(
                                    &field.type_.label,
                                    field.type_.offset,
                                    location + first_instance_location,
                                )
                            })
                            .collect::<Vec<_>>(),
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: None,
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn attribute(type_name: &str, offset: u32, location: usize) -> VertexAttribute {
        VertexAttribute {
            format: match type_name {
                "i32" => VertexFormat::Sint32,
                "u32" => VertexFormat::Uint32,
                "f32" => VertexFormat::Float32,
                "vec2<i32>" => VertexFormat::Sint32x2,
                "vec2<u32>" => VertexFormat::Uint32x2,
                "vec2<f32>" => VertexFormat::Float32x2,
                "vec3<i32>" => VertexFormat::Sint32x3,
                "vec3<u32>" => VertexFormat::Uint32x3,
                "vec3<f32>" => VertexFormat::Float32x3,
                "vec4<i32>" => VertexFormat::Sint32x4,
                "vec4<u32>" => VertexFormat::Uint32x4,
                "vec4<f32>" => VertexFormat::Float32x4,
                _ => unreachable!("internal error: vertex field type should be validated"),
            },
            offset: offset.into(),
            shader_location: location as u32,
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn create_bind_group_layout(
    directive: &Directive,
    module: &Module,
    device: &Device,
    stages: ShaderStages,
) -> BindGroupLayout {
    let storage_entries = module
        .storage_bindings()
        .map(|(_, binding)| BindGroupLayoutEntry {
            binding: binding.index,
            visibility: stages,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage {
                    read_only: binding.is_read_only,
                },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    let uniform_entries = module
        .uniform_bindings()
        .map(|(_, binding)| BindGroupLayoutEntry {
            binding: binding.index,
            visibility: stages,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        });
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(&directive.code()),
        entries: &storage_entries.chain(uniform_entries).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use crate::runner::shaders::RenderShaderResources;
    use wgpu::VertexFormat;

    #[test]
    fn find_attribute() {
        assert_attribute_format("i32", VertexFormat::Sint32);
        assert_attribute_format("i32", VertexFormat::Sint32);
        assert_attribute_format("u32", VertexFormat::Uint32);
        assert_attribute_format("f32", VertexFormat::Float32);
        assert_attribute_format("vec2<i32>", VertexFormat::Sint32x2);
        assert_attribute_format("vec2<u32>", VertexFormat::Uint32x2);
        assert_attribute_format("vec2<f32>", VertexFormat::Float32x2);
        assert_attribute_format("vec3<i32>", VertexFormat::Sint32x3);
        assert_attribute_format("vec3<u32>", VertexFormat::Uint32x3);
        assert_attribute_format("vec3<f32>", VertexFormat::Float32x3);
        assert_attribute_format("vec4<i32>", VertexFormat::Sint32x4);
        assert_attribute_format("vec4<u32>", VertexFormat::Uint32x4);
        assert_attribute_format("vec4<f32>", VertexFormat::Float32x4);
    }

    fn assert_attribute_format(type_name: &str, expected_format: VertexFormat) {
        assert_eq!(
            RenderShaderResources::attribute(type_name, 0, 0).format,
            expected_format
        );
    }
}
