
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vec3<f32>(1.0, 1.0, 0.0);
    out.clip_position = vec4<f32>(
        f32(1 - i32(index)) * 0.5,
        f32(i32(index & 1u) * 2 - 1) * 0.5,
        0.0,
        1.0
    );
    return out;
}
 

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}