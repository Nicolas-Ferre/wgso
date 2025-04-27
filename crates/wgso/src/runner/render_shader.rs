use crate::directive::Directive;
use crate::module::Module;
use crate::type_::Type;
use wgpu::{
    BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, CompareFunction,
    DepthBiasState, DepthStencilState, Device, FrontFace, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderStages, StencilState, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat,
    VertexState, VertexStepMode,
};

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
        let layout = (module.binding_count() > 0)
            .then(|| Self::create_bind_group_layout(directive, module, device));
        let pipeline =
            Self::create_pipeline(module, directive, texture_format, device, layout.as_ref());
        Self { pipeline, layout }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_layout(
        directive: &Directive,
        module: &Module,
        device: &Device,
    ) -> BindGroupLayout {
        let storage_entries = module
            .storage_bindings()
            .map(|(_, binding)| BindGroupLayoutEntry {
                binding: binding.index,
                visibility: ShaderStages::VERTEX_FRAGMENT,
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
                visibility: ShaderStages::VERTEX_FRAGMENT,
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
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some(&directive_code),
            layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&directive_code),
                bind_group_layouts: &layout.map_or(vec![], |layout| vec![layout]),
                push_constant_ranges: &[],
            })),
            vertex: VertexState {
                module: &module,
                entry_point: None,
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[VertexBufferLayout {
                    array_stride: vertex_type.size.into(),
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_type
                        .fields
                        .values()
                        .enumerate()
                        .map(|(location, field_type)| Self::attribute(field_type, location))
                        .collect::<Vec<_>>(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: None,
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
    fn attribute(field_type: &Type, location: usize) -> VertexAttribute {
        VertexAttribute {
            format: match field_type.label.as_str() {
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
            offset: field_type.offset.into(),
            shader_location: location as u32,
        }
    }
}
