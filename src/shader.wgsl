struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}
 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (sign(in.uv.y) * in.uv.x * in.uv.x > in.uv.y) {
        discard;
    }
    return vec4<f32>(1.0, 1.0, 0.0, 1.0);
}