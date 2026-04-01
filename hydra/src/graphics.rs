use std::{fs::File, io::BufReader, path::Path, sync::Arc};

use png::Transformations;
use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

pub struct Graphics {
    window: Arc<winit::window::Window>,
    device: Device,
    queue: Queue,
    size_buffer: Buffer,
    surface: Surface<'static>,
    surface_format: TextureFormat,
    clear_color: Color,

    bind_groups: Vec<BindGroup>,
    textures: Vec<Texture>,

    render_pipeline: RenderPipeline,
}

impl Graphics {
    pub async fn new(window: std::sync::Arc<Window>) -> Graphics {
        let size = window.inner_size();
        let instance = Instance::new(&InstanceDescriptor::from_env_or_default());
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];
        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await.unwrap();

        let texture_bind_group_layout = Self::get_default_bind_group_layout(&device);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("../../shader/default.wgsl").into()),
        });
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let size_buffer = device.create_buffer(&BufferDescriptor{
            label: Some("Buffer"),
            size: 16,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let clear_color = Color::WHITE;

        let mut result = Graphics {
            window,
            device,
            queue,
            size_buffer,
            surface,
            surface_format,
            clear_color,
            bind_groups: Vec::new(),
            textures: Vec::new(),
            render_pipeline,
        };
        result.init_bind_group(size.width, size.height);
        result.configure_surface();
        result.clear_screen_texture();
        return result;
    }

    fn configure_surface(&self) {
        self.surface.configure(&self.device, &SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: CompositeAlphaMode::Auto,
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::AutoVsync,
        });
    }

    pub fn render(&self) {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let texture_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            // Without add_srgb_suffix() the image we will be working with
            // might not be "gamma correct".
            format: Some(self.surface_format.add_srgb_suffix()),
            ..Default::default()
        });
        let mut command_encoder = self.device.create_command_encoder(&Default::default());
        let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        for (index, bind_group) in self.bind_groups.iter().enumerate() {
            render_pass.set_bind_group(index as u32, bind_group, &[]);
        }

        render_pass.draw(0..3, 0..1);

        drop(render_pass);

        // Present to screen
        self.queue.submit([command_encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn resize(&self) {
        self.configure_surface();
        self.update_size_buffer();
    }

    pub fn update_size_buffer(&self) {
        let inner_size = self.window.inner_size();
        let texture = &self.textures[0];

        let width = inner_size.width as f32;
        let height = inner_size.height as f32;
        let aspect_ratio = texture.width() as f32 / texture.height() as f32;

        let new_buffer = [width.to_ne_bytes(), height.to_ne_bytes(), aspect_ratio.to_ne_bytes(), [0; 4]].concat();
        self.queue.write_buffer(&self.size_buffer, 0, &new_buffer);
    }

    fn get_default_bind_group_layout(device: &Device) -> BindGroupLayout {
        return device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Uniform, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Bind Group Layout (Texture)"),
        });
    }

    fn init_bind_group(&mut self, width: u32, height: u32) {
        let screen_texture = self.device.create_texture(&TextureDescriptor {
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("Screen Texture"),
            view_formats: &[],
        });
        let screen_texture_view = screen_texture.create_view(&TextureViewDescriptor::default());
        let screen_sampler = self.device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout = Self::get_default_bind_group_layout(&self.device);
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(self.size_buffer.as_entire_buffer_binding()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&screen_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&screen_sampler),
                },
            ],
            label: Some("Bind Group (Screen Texture)"),
        });
        
        // Use texture index 0 as a screen texture
        self.bind_groups.clear();
        self.bind_groups.insert(0, bind_group);
        self.textures.clear();
        self.textures.insert(0, screen_texture);
    }

    pub fn init_emulator(&mut self, width: u32, height: u32) {
        self.resize_screen_texture(width, height);
        self.clear_color = Color::BLACK;
    }

    pub fn resize_screen_texture(&mut self, width: u32, height: u32) {
        self.init_bind_group(width, height);
        self.update_size_buffer();
    }

    pub fn update_screen_texture(&self, new_buffer: &[u8]) {
        // Use texture index 0 as a screen texture
        let screen_texture = &self.textures[0];
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: screen_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            new_buffer,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * screen_texture.size().width),
                rows_per_image: Some(screen_texture.size().height),
            },
            Extent3d {
                width: screen_texture.size().width,
                height: screen_texture.size().height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn clear_screen_texture(&mut self) {
        let logo_file = File::open(Path::new("images/logo.png")).unwrap();
        let mut decoder = png::Decoder::new(BufReader::new(logo_file));
        let info = decoder.read_header_info().unwrap();
        let width = info.width;
        let height = info.height;
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0xFF; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut buf).unwrap();

        for [r, g, b, a] in buf.as_chunks_mut().0 {
            *r += (((0xFF - *r) as u16 * (0xFF - *a) as u16) / 255) as u8;
            *g += (((0xFF - *g) as u16 * (0xFF - *a) as u16) / 255) as u8;
            *b += (((0xFF - *b) as u16 * (0xFF - *a) as u16) / 255) as u8;
        }

        self.resize_screen_texture(width, height);
        self.update_screen_texture(&buf);
        self.clear_color = Color::WHITE;
    }
}
