use std::sync::Arc;

use wgpu::{Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, RenderPassDescriptor, RequestAdapterOptions, SurfaceConfiguration, TextureUsages, TextureViewDescriptor};
use winit::window::Window;

pub struct Graphics {
    window: Arc<winit::window::Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
}

impl Graphics {
    pub async fn new(window: std::sync::Arc<Window>) -> Graphics {
        let size = window.inner_size();
        let instance_descriptor = InstanceDescriptor::from_env_or_default();
        let instance = Instance::new(&instance_descriptor);
        let surface = instance.create_surface(window.clone()).unwrap();
        let request_adapter_options = RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface)  
        };
        let adapter = instance.request_adapter(&request_adapter_options).await.unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities.formats[0];
        let device_descriptor = DeviceDescriptor::default();
        let (device, queue) = adapter.request_device(&device_descriptor).await.unwrap();

        let result = Graphics {window, device, queue, size, surface, surface_format};
        result.configure_surface();
        return result;
    }

    pub fn configure_surface(&self) {
        let surface_configuration = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.device, &surface_configuration);
    }

    pub fn render_start(&self) {
        // Render test
        let surface_texture = self.surface.get_current_texture().unwrap();
        let texture_view_descriptor = TextureViewDescriptor {
            // Without add_srgb_suffix() the image we will be working with
            // might not be "gamma correct".
            format: Some(self.surface_format.add_srgb_suffix()),
            ..Default::default()
        };
        let texture_view = surface_texture.texture.create_view(&texture_view_descriptor);
        let mut command_encoder = self.device.create_command_encoder(&Default::default());
        let render_pass_descriptor = RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };
        let render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);

        // RENDER CODE HERE
        

        drop(render_pass);
        self.queue.submit([command_encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}