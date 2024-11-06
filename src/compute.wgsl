struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> input_points: array<vec3<f32>>;
// [s, a, b], [c, 0, 0], 
// [x0, y0, z0], [x1, y1, z1], [x2, y2, z2]
@group(0) @binding(1)
var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(2)
var<storage, read_write> indices: array<u32>;
// 0, 1, 2, 3, 4
// 0, 1, 2, 2, 3, 4
// [0, 1, 2] [0, 0, 2]
// [2, 3, 4] [0, 2, 4]
// [4, 5, 6] [0, 4, 6]

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x * 2u;
    let j = id.x * 3u;
    let k = id.x * 6u;

    if (i + 2u >= arrayLength(&input_points)) {
        return;
    }
    let v0 = input_points[i];
    let v1 = input_points[i + 1u];
    let v2 = input_points[i + 2u];

    vertices[j] = Vertex(v0, vec2<f32>(0.0, 0.0));
    vertices[j + 1] = Vertex(v1, vec2<f32>(0.5, 0.0));
    vertices[j + 2] = Vertex(v2, vec2<f32>(1.0, select(-1.0, 1.0, cross(v1 - v0, v2 - v1).z > 0)));

    indices[k] = j;
    indices[k + 1] = j + 1;
    indices[k + 2] = j + 2;
    indices[k + 3] = 0u;
    indices[k + 4] = j;
    indices[k + 5] = j + 2;
}
