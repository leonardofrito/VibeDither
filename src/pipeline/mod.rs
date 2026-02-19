use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use image::GenericImageView;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorSettings {
    // Block 1: Basic Light
    pub exposure: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    // Block 2: White Balance & Extra Light
    pub whites: f32,
    pub blacks: f32,
    pub temperature: f32,
    pub tint: f32,
    // Block 3: Color & Detail
    pub saturation: f32,
    pub vibrance: f32,
    pub sharpness: f32,
    pub brightness: f32,
    // Block 4: Dither Base
    pub dither_enabled: f32,
    pub dither_type: f32,
    pub dither_scale: f32,
    pub dither_threshold: f32,
    // Block 5: Dither Style
    pub dither_color: f32,
    pub posterize_levels: f32,
    pub bayer_size: f32, // 2 to 8
    pub grad_enabled: f32,
}

impl Default for ColorSettings {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 1.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            temperature: 0.0,
            tint: 0.0,
            saturation: 1.0,
            vibrance: 0.0,
            sharpness: 0.0,
            brightness: 0.0,
            dither_enabled: 0.0,
            dither_type: 0.0,
            dither_scale: 1.0,
            dither_threshold: 0.5,
            dither_color: 0.0,
            posterize_levels: 0.0,
            bayer_size: 8.0,
            grad_enabled: 0.0,
        }
    }
}

pub struct Pipeline {
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,
    pub bind_group: Option<wgpu::BindGroup>, // New: Cached bind group
    pub uniform_buffer: Option<wgpu::Buffer>, // New: Uniform buffer
    pub sampler: Option<wgpu::Sampler>,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub curves_texture: Option<wgpu::Texture>,
    pub curves_view: Option<wgpu::TextureView>,
    pub gradient_texture: Option<wgpu::Texture>,
    pub gradient_view: Option<wgpu::TextureView>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [1.0, -1.0], tex_coords: [1.0, 1.0] },
];

impl Pipeline {
    pub fn new() -> Self {
        Self {
            pipeline: None,
            bind_group_layout: None,
            bind_group: None,
            uniform_buffer: None,
            sampler: None,
            vertex_buffer: None,
            curves_texture: None,
            curves_view: None,
            gradient_texture: None,
            gradient_view: None,
        }
    }

    pub fn init(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));

        // Create 1D Curves texture (256x1)
        let curves_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("curves_texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let curves_view = curves_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create 1D Gradient texture (256x1)
        let gradient_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("gradient_texture"),
            size: wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let gradient_view = gradient_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        // Initial uniform buffer (empty for now, will be updated in render)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("settings_uniform_buffer"),
            size: std::mem::size_of::<ColorSettings>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create an initial bind group. This will be updated later with actual texture views.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&curves_view), // Placeholder
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&curves_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&gradient_view),
                },
            ],
            label: Some("initial_bind_group"),
        });


        self.pipeline = Some(render_pipeline);
        self.bind_group_layout = Some(bind_group_layout);
        self.sampler = Some(sampler);
        self.vertex_buffer = Some(vertex_buffer);
        self.curves_texture = Some(curves_texture);
        self.curves_view = Some(curves_view);
        self.gradient_texture = Some(gradient_texture);
        self.gradient_view = Some(gradient_view);
        self.uniform_buffer = Some(uniform_buffer); // Cache uniform buffer
        self.bind_group = Some(bind_group); // Cache bind group
    }

    pub fn update_curves(&self, queue: &wgpu::Queue, data: &[u8; 1024]) {
        if let Some(texture) = &self.curves_texture {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(1024),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub fn update_gradient(&self, queue: &wgpu::Queue, data: &[u8; 1024]) {
        if let Some(texture) = &self.gradient_texture {
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(1024),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub fn create_texture_from_image(&self, device: &wgpu::Device, queue: &wgpu::Queue, img: &image::DynamicImage) -> wgpu::Texture {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("input_texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        texture
    }

    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        input_texture_view: &wgpu::TextureView,
        output_texture_view: &wgpu::TextureView,
        settings: &ColorSettings,
    ) {
        let pipeline = self.pipeline.as_ref().unwrap();
        let bind_group_layout = self.bind_group_layout.as_ref().unwrap();
        let sampler = self.sampler.as_ref().unwrap();
        let vertex_buffer = self.vertex_buffer.as_ref().unwrap();
        let curves_view = self.curves_view.as_ref().unwrap();
        let gradient_view = self.gradient_view.as_ref().unwrap();
        let uniform_buffer = self.uniform_buffer.as_ref().unwrap(); // Use cached uniform buffer

        // Update the uniform buffer with new settings
        queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[*settings]));

        // Recreate bind group only if necessary (e.g., input texture changes)
        // For simplicity here, we'll just recreate it always to match the previous behavior
        // in main.rs ensuring that the input_texture_view is correctly bound each time.
        // A more optimized approach would cache this and only recreate if input_texture_view's
        // underlying texture changes.
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(curves_view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(gradient_view),
                },
            ],
            label: Some("bind_group"),
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..4, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
