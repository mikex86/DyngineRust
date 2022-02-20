struct CameraUniform {
    view_proj: mat4x4<f32>;
};
[[group(0), binding(0)]] // 1.
var<uniform> camera: CameraUniform;

struct VSOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VSOutput {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    let color = vec4<f32>(
      f32(i32(in_vertex_index) % 3),
      f32((i32(in_vertex_index) + 1) % 3),
      f32((i32(in_vertex_index) + 2) % 3),
      1.0
    );
    return VSOutput(camera.view_proj * vec4<f32>(x, y, 0.0, 1.0), color);
}

[[stage(fragment)]]
fn fs_main(in: VSOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}