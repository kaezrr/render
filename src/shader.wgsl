struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32
) -> VertexOutput {
    var out: VertexOutput;

    let xs = array<f32, 3>(0.5, 0.0, -0.5);
    let ys = array<f32, 3>(-0.5, 0.5, -0.5);

    out.clip_position = vec4(
        xs[in_vertex_index],
        ys[in_vertex_index],
        0.0,
        1.0
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0)vec4<f32> {
    return vec4(0.8, 0.4, 0.5, 1.0);
}
