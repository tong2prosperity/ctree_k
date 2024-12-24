#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SnowflakeVertex {
    position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SnowflakeInstance {
    position: [f32; 3],
    velocity: [f32; 3],
    size: f32,
    alpha: f32,
}

pub struct SnowfallSystem {
    instances: Vec<SnowflakeInstance>,
    instance_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    max_snowflakes: usize,
}

impl SnowfallSystem {
    pub fn new(max_snowflakes: usize) -> Self {
        Self {
            snowflakes: Vec::new(),
            max_snowflakes,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // 生成新的雪花
        if self.snowflakes.len() < self.max_snowflakes {
            let new_snowflake = Snowflake {
                position: Vector3::from_xyz(
                    rand::random::<f32>() * 20.0 - 10.0, // x范围 [-10, 10]
                    10.0,                                // 从顶部开始
                    rand::random::<f32>() * 20.0 - 10.0, // z范围 [-10, 10]
                ),
                velocity: Vector3::from_xyz(
                    rand::random::<f32>() * 0.5 - 0.25, // 随机x方向速度
                    -1.0 - rand::random::<f32>(),       // 向下落的速度
                    rand::random::<f32>() * 0.5 - 0.25, // 随机z方向速度
                ),
                size: rand::random::<f32>() * 0.2 + 0.1,
                alpha: rand::random::<f32>() * 0.5 + 0.5,
            };
            self.snowflakes.push(new_snowflake);
        }

        // 更新现有雪花
        self.snowflakes.retain_mut(|snowflake| {
            snowflake.position = snowflake.position + snowflake.velocity * dt;
            snowflake.position.y() > -10.0 // 当雪花落到底部时移除
        });
    }
}
