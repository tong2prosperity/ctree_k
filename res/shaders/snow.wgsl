struct VertexInput {
    @location(0) vertex_position: vec3<f32>,  // 改名为 vertex_position
};

struct InstanceInput {
    @location(1) instance_position: vec3<f32>,  // 改名为 instance_position
    @location(2) velocity: vec3<f32>,
    @location(3) size: f32,
    @location(4) alpha: f32,
};

struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) alpha: f32,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let world_position = vec4<f32>(
        vertex.vertex_position * instance.size + instance.instance_position,  // 使用新的名称
        1.0
    );
    out.clip_position = camera.view_proj * world_position;
    out.alpha = instance.alpha;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, in.alpha);
}