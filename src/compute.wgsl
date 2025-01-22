struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
};

@group(0) @binding(0)
var<storage, read> points: array<vec3<f32>>;
@group(0) @binding(1)
var<storage, read_write> vertices: array<Vertex>;
@group(0) @binding(2)
var<storage, read_write> indices: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x * 2u;
    let len = arrayLength(&points);
    if i + 2u >= len { return; }
    let j = id.x * 3u;
    let k = id.x * 6u;

    vertices[j] = Vertex(points[i], vec2<f32>(0.0, 0.0));
    vertices[j + 1] = Vertex(points[i + 1u], vec2<f32>(0.5, 0.0));
    vertices[j + 2] = Vertex(points[i + 2u], vec2<f32>(1.0, 1.0));

    // Edge triangle, that will be clipped with quadratic test
    indices[k] = j;
    indices[k + 1] = j + 1;
    indices[k + 2] = j + 2;

    // Filled triangle fan
    indices[k + 3] = 0u;
    indices[k + 4] = j;
    indices[k + 5] = j + 3;

    if i == len - 3 {
        // If last point = first point, close the loop
        if points[0].x == points[len - 1].x && points[0].y == points[len - 1].y && points[0].z == points[len - 1].z {
            indices[k + 5] = 0u;
        } else {
            // Otherwise, add the last vertex at j + 3 for the last triangle
            // indices[k + 5] is already set to j + 3
            vertices[j + 3] = Vertex(points[i + 2u], vec2<f32>(0.0, 0.0));
        }
    }
}
