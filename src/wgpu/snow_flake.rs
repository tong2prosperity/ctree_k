use std::{mem, time::Duration};

use super::{model::Vertex, texture};
use pixels::wgpu;
use pixels::wgpu::{util::DeviceExt as _, Device};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SnowflakeVertex {
    pub position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SnowflakeInstance {
    position: [f32; 3],
    velocity: [f32; 3],
    size: f32,
    alpha: f32,
}

impl SnowflakeInstance {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SnowflakeInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // velocity
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // size
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
                // alpha
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

impl Vertex for SnowflakeVertex {
    fn desc<'a>() -> pixels::wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SnowflakeVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

pub struct SnowfallSystem {
    snowflakes: Vec<SnowflakeInstance>,
    instance_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    max_snowflakes: usize,
}

impl SnowfallSystem {
    pub fn new(device: &Device, max_snowflakes: usize) -> Self {
        // Create basic snowflake vertex (just a point)
        let vertices = vec![SnowflakeVertex {
            position: [0.0, 0.0, 0.0],
        }];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Snowflake Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create empty instance buffer
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Snowflake Instance Buffer"),
            size: (std::mem::size_of::<SnowflakeInstance>() * max_snowflakes)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Snow Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/snow.wgsl").into()),
        });

        let render_pipeline = create_snow_pipeline(device, &shader);

        Self {
            snowflakes: Vec::new(),
            instance_buffer,
            vertex_buffer,
            render_pipeline,
            max_snowflakes,
        }
    }

    pub fn render<'a>(
        &'a self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        camera_bind_group: &'a wgpu::BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Snow Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // 使用load而不是clear,这样不会清除之前绘制的内容
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    // 同样使用load保留之前的深度信息
                    load: wgpu::LoadOp::Load,
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..1, 0..self.snowflakes.len() as u32);
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: Duration) {
        let dt = dt.as_secs_f32();

        // Generate new snowflakes
        if self.snowflakes.len() < self.max_snowflakes {
            let new_snowflake = SnowflakeInstance {
                position: [
                    rand::random::<f32>() * 20.0 - 10.0,
                    10.0,
                    rand::random::<f32>() * 20.0 - 10.0,
                ],
                velocity: [
                    rand::random::<f32>() * 0.5 - 0.25,
                    -1.0 - rand::random::<f32>(),
                    rand::random::<f32>() * 0.5 - 0.25,
                ],
                size: rand::random::<f32>() * 0.2 + 0.1,
                alpha: rand::random::<f32>() * 0.5 + 0.5,
            };
            self.snowflakes.push(new_snowflake);
        }

        // Update existing snowflakes
        self.snowflakes.retain_mut(|snowflake| {
            snowflake.position[0] += snowflake.velocity[0] * dt;
            snowflake.position[1] += snowflake.velocity[1] * dt;
            snowflake.position[2] += snowflake.velocity[2] * dt;
            snowflake.position[1] > -10.0 // Remove when below certain height
        });

        // Update instance buffer
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.snowflakes),
        );
    }

    // pub fn update(&mut self, dt: f32) {
    //     // 生成新的雪花
    //     if self.snowflakes.len() < self.max_snowflakes {
    //         let new_snowflake = SnowflakeInstance {
    //             position: Vector3::from_xyz(
    //                 rand::random::<f32>() * 20.0 - 10.0, // x范围 [-10, 10]
    //                 10.0,                                // 从顶部开始
    //                 rand::random::<f32>() * 20.0 - 10.0, // z范围 [-10, 10]
    //             ),
    //             velocity: Vector3::from_xyz(
    //                 rand::random::<f32>() * 0.5 - 0.25, // 随机x方向速度
    //                 -1.0 - rand::random::<f32>(),       // 向下落的速度
    //                 rand::random::<f32>() * 0.5 - 0.25, // 随机z方向速度
    //             ),
    //             size: rand::random::<f32>() * 0.2 + 0.1,
    //             alpha: rand::random::<f32>() * 0.5 + 0.5,
    //         };
    //         self.snowflakes.push(new_snowflake);
    //     }

    //     // 更新现有雪花
    //     self.snowflakes.retain_mut(|snowflake| {
    //         snowflake.position = snowflake.position + snowflake.velocity * dt;
    //         snowflake.position.y() > -10.0 // 当雪花落到底部时移除
    //     });
    // }

    // pub fn init(&mut self, device: &Device) {
    //     self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Snowflake Instance Buffer"),
    //         contents: bytemuck::cast_slice(&self.snowflakes),
    //         usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    //     });

    //     let snow_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    //         label: Some("Snow Shader"),
    //         source: wgpu::ShaderSource::Wgsl(include_str!("../../res/shaders/snow.wgsl").into()),
    //     });

    //     let num_snowflakes = 1000;
    // let mut snows = Vec::new();
    //         for _ in 0..num_snowflakes {
    // // 生成随机位置和大小
    //             let x = rand::random::<f32>() * 2.0 - 1.0;
    // let y = rand::random::<f32>() * 2.0 - 1.0;
    // let z = rand::random::<f32>() * 2.0 - 1.0;
    //         snows.push(SnowflakeVertex { position: [x, y, z] });
    //         // 根据需要添加更多顶点以形成雪花形状
    //     }
    //     let snow_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //         label: Some("Vertex Buffer"),
    //         contents: bytemuck::cast_slice(&snows),
    //         usage: wgpu::BufferUsages::VERTEX,
    //     });

    //     let snow_layout = wgpu::VertexBufferLayout {
    //         array_stride: std::mem::size_of::<SnowflakeVertex>() as wgpu::BufferAddress,
    //         step_mode: wgpu::VertexStepMode::Vertex,
    //         attributes: &[
    //             wgpu::VertexAttribute {
    //                 offset: 0,
    //                 shader_location: 0,
    //                 format: wgpu::VertexFormat::Float32x3,
    //             },
    //         ],
    //     };

    //     let snow_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    //         label: Some("snow_bindgroup_layout"),
    //         entries: &[wgpu::BindGroupLayoutEntry {
    //             binding: 0,
    //             visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
    //             ty: wgpu::BindingType::Buffer {
    //                 ty: wgpu::BufferBindingType::Uniform,
    //                 has_dynamic_offset: false,
    //                 min_binding_size: None,
    //             },
    //             count: None,
    //         }],
    //     });

    //     let snow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
    //         layout: &snow_bindgroup_layout,
    //         entries: &[wgpu::BindGroupEntry {
    //             binding: 0,
    //             resource: snow_vertex_buffer.as_entire_binding(),
    //         }],
    //         label: Some("snow_bind_group"),
    //     });

    // }
}

fn create_snow_pipeline(device: &Device, shader: &wgpu::ShaderModule) -> wgpu::RenderPipeline {
    // 创建相机绑定组布局
    let camera_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Snow Pipeline Layout"),
        bind_group_layouts: &[&camera_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Snow Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[SnowflakeVertex::desc(), SnowflakeInstance::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::PointList,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
