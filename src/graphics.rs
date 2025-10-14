use std::sync::Arc;

use muda::dpi::PhysicalSize;
use wgpu::*;
use winit::window::Window;

pub struct Graphics {
    window: Arc<winit::window::Window>,
    device: Device,
    queue: Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: Surface<'static>,
    surface_format: TextureFormat,

    bind_group: BindGroup,
    screen_texture: Texture,

    render_pipeline: RenderPipeline,
}

impl Graphics {
    pub async fn new(window: std::sync::Arc<Window>, opt_size: Option<PhysicalSize<u32>>) -> Graphics {
        let size = match opt_size {
            Some(s) => s,
            None => window.inner_size(),
        };
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

        let (screen_texture, bind_group) = Self::bind_screen_texture(&device, size.width, size.height);
        let texture_bind_group_layout = Self::get_default_bind_group_layout(&device);

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("../shader/test.wgsl").into()),
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
                    blend: Some(BlendState::REPLACE),
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

        let result = Graphics {
            window,
            device,
            queue,
            size,
            surface,
            surface_format,
            bind_group,
            screen_texture,
            render_pipeline,
        };
        result.configure_surface();
        return result;
    }

    fn configure_surface(&self) {
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: CompositeAlphaMode::Auto,
            width: self.window.inner_size().width,
            height: self.window.inner_size().height,
            desired_maximum_frame_latency: 2,
            present_mode: PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_configuration);
    }

    pub fn render(&self) {
        self.window.request_redraw(); 

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
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        render_pass.draw(0..3, 0..1);

        drop(render_pass);
        
        // Present to screen
        self.queue.submit([command_encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }

    pub fn resize(&self) {
        self.configure_surface();
    }

    fn get_default_bind_group_layout(device: &Device) -> BindGroupLayout {
        return device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Bind Group Layout (Texture)"),
        });
    }

    fn bind_screen_texture(device: &Device, width: u32, height: u32) -> (Texture, BindGroup) {
        let screen_texture = device.create_texture(&TextureDescriptor {
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
        let screen_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group_layout = Self::get_default_bind_group_layout(device);
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&screen_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&screen_sampler),
                },
            ],
            label: Some("Bind Group (Screen Texture)"),
        });
        return (screen_texture, bind_group);
    }

    pub fn resize_screen_texture(&mut self, width: u32, height: u32) {
        (self.screen_texture, self.bind_group) = Self::bind_screen_texture(&self.device, width, height);
    }

    pub fn update_screen_texture(&self, new_buffer: &[u8]) {
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.screen_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &new_buffer,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.screen_texture.size().width),
                rows_per_image: Some(self.screen_texture.size().height),
            },
            Extent3d {
                width: self.screen_texture.size().width,
                height: self.screen_texture.size().height,
                depth_or_array_layers: 1,
            },
        );
    }
}
