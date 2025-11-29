use std::collections::HashMap;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use rgb_core::{CHUNK_SIZE, ChunkPos, Color, REGION_SIZE};

const ATLAS_SIZE: u32 = 256; // 256x256 chunks = 65536 max chunks
const ATLAS_PIXELS: u32 = ATLAS_SIZE * CHUNK_SIZE as u32;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ChunkInstance {
    pub world_pos: [f32; 2],
    pub atlas_uv: [f32; 2],
    pub region_color: [f32; 3],
    pub chunk_in_region: [f32; 2],
    pub _padding: f32, // Align to 12 floats (48 bytes)
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

pub struct ChunkAtlas {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub allocations: HashMap<ChunkPos, (u32, u32)>,
    free_slots: Vec<(u32, u32)>,
    next_slot: (u32, u32),
}

impl ChunkAtlas {
    fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Chunk Atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_PIXELS,
                height: ATLAS_PIXELS,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            texture_view,
            allocations: HashMap::new(),
            free_slots: Vec::new(),
            next_slot: (0, 0),
        }
    }

    pub fn allocate(&mut self, pos: ChunkPos) -> (u32, u32) {
        if let Some(&slot) = self.allocations.get(&pos) {
            return slot;
        }

        let slot = if let Some(slot) = self.free_slots.pop() {
            slot
        } else {
            let slot = self.next_slot;
            self.next_slot.0 += 1;
            if self.next_slot.0 >= ATLAS_SIZE {
                self.next_slot.0 = 0;
                self.next_slot.1 += 1;
            }
            slot
        };

        self.allocations.insert(pos, slot);
        slot
    }

    #[allow(dead_code)]
    pub fn deallocate(&mut self, pos: ChunkPos) {
        if let Some(slot) = self.allocations.remove(&pos) {
            self.free_slots.push(slot);
        }
    }

    #[allow(dead_code)]
    pub fn get_slot(&self, pos: ChunkPos) -> Option<(u32, u32)> {
        self.allocations.get(&pos).copied()
    }
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub atlas: ChunkAtlas,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub atlas_bind_group: wgpu::BindGroup,
    pub instance_count: u32,
    pub camera_uniform: CameraUniform,
}

impl Renderer {
    pub async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        let atlas = ChunkAtlas::new(&device);

        // Create shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Chunk Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/chunk.wgsl").into()),
        });

        // Camera uniform buffer
        let camera_uniform = CameraUniform {
            view_proj: orthographic_projection(0.0, size.width as f32, 0.0, size.height as f32),
        };
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Atlas bind group
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Atlas Bind Group Layout"),
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

        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &atlas_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    // Vertex buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
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
                    },
                    // Instance buffer
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<ChunkInstance>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // world_pos
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // atlas_uv
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 3,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            // region_color
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 4,
                                format: wgpu::VertexFormat::Float32x3,
                            },
                            // chunk_in_region
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                                shader_location: 5,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // Quad vertices: position (x, y), tex_coord (u, v)
        let vertices: [[f32; 4]; 4] = [
            [0.0, 0.0, 0.0, 1.0], // bottom-left
            [1.0, 0.0, 1.0, 1.0], // bottom-right
            [1.0, 1.0, 1.0, 0.0], // top-right
            [0.0, 1.0, 0.0, 0.0], // top-left
        ];

        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Instance buffer (allocate space for many chunks)
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: std::mem::size_of::<ChunkInstance>() as u64 * 65536,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline,
            atlas,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            camera_buffer,
            camera_bind_group,
            atlas_bind_group,
            instance_count: 0,
            camera_uniform,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    pub fn update_camera(&mut self, position: [f32; 2], zoom: f32) {
        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;

        let half_width = width * 0.5 / zoom;
        let half_height = height * 0.5 / zoom;

        self.camera_uniform.view_proj = orthographic_projection(
            position[0] - half_width,
            position[0] + half_width,
            position[1] - half_height,
            position[1] + half_height,
        );

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn upload_chunk_texture(&self, slot: (u32, u32), cells: &[[bool; CHUNK_SIZE]; CHUNK_SIZE]) {
        let mut data = vec![0u8; CHUNK_SIZE * CHUNK_SIZE * 4];

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = (y * CHUNK_SIZE + x) * 4;
                if cells[y][x] {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                    data[idx + 3] = 255;
                } else {
                    data[idx] = 20;
                    data[idx + 1] = 20;
                    data[idx + 2] = 20;
                    data[idx + 3] = 255;
                }
            }
        }

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.atlas.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: slot.0 * CHUNK_SIZE as u32,
                    y: slot.1 * CHUNK_SIZE as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(CHUNK_SIZE as u32 * 4),
                rows_per_image: Some(CHUNK_SIZE as u32),
            },
            wgpu::Extent3d {
                width: CHUNK_SIZE as u32,
                height: CHUNK_SIZE as u32,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn update_instances(&mut self, instances: &[ChunkInstance]) {
        self.instance_count = instances.len() as u32;
        if !instances.is_empty() {
            self.queue
                .write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.atlas_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..self.instance_count);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn orthographic_projection(left: f32, right: f32, bottom: f32, top: f32) -> [[f32; 4]; 4] {
    let tx = -(right + left) / (right - left);
    let ty = -(top + bottom) / (top - bottom);

    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [tx, ty, 0.0, 1.0],
    ]
}

pub const CELL_SIZE: f32 = 4.0;
pub const CHUNK_PIXEL_SIZE: f32 = CHUNK_SIZE as f32 * CELL_SIZE;

pub fn chunk_world_pos(pos: ChunkPos) -> [f32; 2] {
    [
        pos.x as f32 * CHUNK_PIXEL_SIZE,
        pos.y as f32 * CHUNK_PIXEL_SIZE,
    ]
}

pub fn atlas_uv(slot: (u32, u32)) -> [f32; 2] {
    [
        slot.0 as f32 / ATLAS_SIZE as f32,
        slot.1 as f32 / ATLAS_SIZE as f32,
    ]
}

/// Get RGB color for a region's color
pub fn region_color_rgb(color: Color) -> [f32; 3] {
    match color {
        Color::C0 => [1.0, 0.3, 0.3], // Red
        Color::C1 => [0.3, 1.0, 0.3], // Green
        Color::C2 => [0.3, 0.3, 1.0], // Blue
        Color::C3 => [1.0, 1.0, 0.3], // Yellow
        Color::C4 => [1.0, 0.3, 1.0], // Magenta
        Color::C5 => [0.3, 1.0, 1.0], // Cyan
        Color::C6 => [1.0, 0.6, 0.3], // Orange
        Color::C7 => [0.6, 0.3, 1.0], // Purple
        Color::C8 => [0.3, 1.0, 0.6], // Teal
    }
}

/// Get the position of a chunk within its region (0-3 for x and y)
pub fn chunk_in_region(pos: ChunkPos) -> [f32; 2] {
    [
        pos.x.rem_euclid(REGION_SIZE as i32) as f32,
        pos.y.rem_euclid(REGION_SIZE as i32) as f32,
    ]
}
