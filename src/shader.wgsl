struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ObjectUniforms {
    model: mat4x4<f32>,
    color: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> uniforms: ObjectUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) col: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = camera.view_proj * uniforms.model * vec4<f32>(model.position, 1.0);
    out.col = uniforms.color;
    // out.col = vec4<f32>(vec3<f32>(rand(model.uv + model.position.xy)), 1.0);
    return out;
}
 
 fn rand(n: vec2<f32>) -> f32 {
    return fract(sin(dot(n, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn stencil(in: VertexOutput) {
    if (in.uv.x * in.uv.x > in.uv.y) {
        discard;
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.col;
}