//! Nearest-neighbor textured quads. One pipeline, one texture bind group per image.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use fablespace_data::Paths;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::font8x8::FONT8X8;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

struct Image {
    bind: wgpu::BindGroup,
    width: u32,
    height: u32,
}

pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    images: HashMap<String, Image>,
    missing: HashMap<String, ()>,
}

pub struct Sprite {
    pub key: String,
    pub src: (u32, u32, u32, u32),
    pub dst: (i32, i32, i32, i32),
    pub color: [f32; 4],
}

impl Gpu {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("create surface: {e}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await;
        let adapter = match adapter {
            Some(adapter) => adapter,
            None => instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: true,
                })
                .await
                .ok_or_else(|| "no graphics adapter".to_string())?,
        };
        eprintln!("graphics: {}", adapter.get_info().name);
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("fablespace"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .map_err(|e| format!("request device: {e}"))?;
        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tex"),
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
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..wgpu::SamplerDescriptor::default()
        });
        let mut gpu = Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            layout,
            sampler,
            images: HashMap::new(),
            missing: HashMap::new(),
        };
        gpu.insert("white", 1, 1, &[255, 255, 255, 255]);
        gpu.insert("font", 1024, 8, &font_pixels());
        Ok(gpu)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn size(&mut self, paths: &Paths, key: &str) -> Option<(u32, u32)> {
        if let Some(image) = self.images.get(key) {
            return Some((image.width, image.height));
        }
        if key == "white" || key == "font" {
            return None;
        }
        let (folder, name) = key.split_once('/')?;
        let path = paths.image(folder, name);
        match load_rgba(&path) {
            Some((w, h, pixels)) => {
                self.insert(key, w, h, &pixels);
                Some((w, h))
            }
            None => {
                if self.missing.insert(key.to_string(), ()).is_none() {
                    eprintln!("missing image {}", path.display());
                }
                None
            }
        }
    }

    pub fn draw(&mut self, sprites: &[Sprite], width: u32, height: u32, view_w: u32, view_h: u32) {
        if width == 0 || height == 0 {
            return;
        }
        if self.config.width != width || self.config.height != height {
            self.resize(width, height);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(err) => {
                eprintln!("surface: {err}");
                return;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("frame"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.06,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            let scale = ((width / view_w.max(1)).min(height / view_h.max(1))).max(1) as f32;
            let view_w = view_w as f32 * scale;
            let view_h = view_h as f32 * scale;
            let ox = (width as f32 - view_w) * 0.5;
            let oy = (height as f32 - view_h) * 0.5;
            let mut start = 0;
            while start < sprites.len() {
                let key = sprites[start].key.as_str();
                let mut end = start + 1;
                while end < sprites.len() && sprites[end].key == key {
                    end += 1;
                }
                if let Some(image) = self.images.get(key) {
                    let batch = &sprites[start..end];
                    let (vertices, indices) = geometry(
                        batch,
                        image.width,
                        image.height,
                        scale,
                        ox,
                        oy,
                        width,
                        height,
                    );
                    if !indices.is_empty() {
                        let vbuf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("verts"),
                                    contents: bytemuck::cast_slice(&vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                });
                        let ibuf =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("indices"),
                                    contents: bytemuck::cast_slice(&indices),
                                    usage: wgpu::BufferUsages::INDEX,
                                });
                        pass.set_bind_group(0, &image.bind, &[]);
                        pass.set_vertex_buffer(0, vbuf.slice(..));
                        pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                        pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
                    }
                }
                start = end;
            }
        }
        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    fn insert(&mut self, key: &str, width: u32, height: u32, rgba: &[u8]) {
        let (padded, bytes_per_row) = pad_rows(width, height, rgba);
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(key),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &padded,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(key),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });
        self.images.insert(
            key.to_string(),
            Image {
                bind,
                width,
                height,
            },
        );
    }
}

fn geometry(
    sprites: &[Sprite],
    tex_w: u32,
    tex_h: u32,
    scale: f32,
    ox: f32,
    oy: f32,
    win_w: u32,
    win_h: u32,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(sprites.len() * 4);
    let mut indices = Vec::with_capacity(sprites.len() * 6);
    let win_w = win_w as f32;
    let win_h = win_h as f32;
    for sprite in sprites {
        if sprite.dst.2 <= 0 || sprite.dst.3 <= 0 {
            continue;
        }
        let x = ox + sprite.dst.0 as f32 * scale;
        let y = oy + sprite.dst.1 as f32 * scale;
        let w = sprite.dst.2 as f32 * scale;
        let h = sprite.dst.3 as f32 * scale;
        let x0 = x / win_w * 2.0 - 1.0;
        let y0 = 1.0 - y / win_h * 2.0;
        let x1 = (x + w) / win_w * 2.0 - 1.0;
        let y1 = 1.0 - (y + h) / win_h * 2.0;
        let (u0, v0, u1, v1) = if sprite.src.2 == 0 || sprite.src.3 == 0 || tex_w == 0 || tex_h == 0
        {
            (0.0, 0.0, 1.0, 1.0)
        } else {
            (
                sprite.src.0 as f32 / tex_w as f32,
                sprite.src.1 as f32 / tex_h as f32,
                (sprite.src.0 + sprite.src.2) as f32 / tex_w as f32,
                (sprite.src.1 + sprite.src.3) as f32 / tex_h as f32,
            )
        };
        let base = vertices.len() as u32;
        let color = sprite.color;
        vertices.push(Vertex {
            pos: [x0, y0],
            uv: [u0, v0],
            color,
        });
        vertices.push(Vertex {
            pos: [x1, y0],
            uv: [u1, v0],
            color,
        });
        vertices.push(Vertex {
            pos: [x1, y1],
            uv: [u1, v1],
            color,
        });
        vertices.push(Vertex {
            pos: [x0, y1],
            uv: [u0, v1],
            color,
        });
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    (vertices, indices)
}

fn pad_rows(width: u32, height: u32, rgba: &[u8]) -> (Vec<u8>, u32) {
    let row = width.saturating_mul(4);
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = row.div_ceil(align) * align;
    let mut out = vec![0u8; padded as usize * height as usize];
    for y in 0..height as usize {
        let src = y * row as usize;
        let dst = y * padded as usize;
        let n = row as usize;
        if src + n <= rgba.len() {
            out[dst..dst + n].copy_from_slice(&rgba[src..src + n]);
        }
    }
    (out, padded)
}

fn load_rgba(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let image = image::open(path).ok()?.to_rgba8();
    Some((image.width(), image.height(), image.into_raw()))
}

fn font_pixels() -> Vec<u8> {
    let mut pixels = vec![0u8; 1024 * 8 * 4];
    for (index, glyph) in FONT8X8.iter().enumerate() {
        for row in 0..8 {
            let bits = glyph[row];
            for col in 0..8 {
                let on = (bits >> col) & 1 == 1;
                let x = index * 8 + col;
                let i = (row * 1024 + x) * 4;
                let v = if on { 255 } else { 0 };
                pixels[i] = v;
                pixels[i + 1] = v;
                pixels[i + 2] = v;
                pixels[i + 3] = v;
            }
        }
    }
    pixels
}

const SHADER: &str = r#"
struct VsIn {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}
struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}
@vertex
fn vs(in: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = vec4<f32>(in.pos, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, in.uv) * in.color;
}
"#;
