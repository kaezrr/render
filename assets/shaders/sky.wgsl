struct Camera {
    view_position: vec4<f32>,
    view: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    inv_projection: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

@group(1) @binding(0)
var env_map: texture_cube<f32>;

@group(1) @binding(1)
var env_sampler: sampler;

struct VertexOutput {
    @builtin(position) frag_position: vec4<f32>,
    @location(0) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) id: u32,
) -> VertexOutput {

    let uv = vec2<f32>(
        f32(id & 1),
        f32((id >> 1) & 1),
    );

    var out: VertexOutput;

    out.clip_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);
    out.frag_position = vec4(uv * 4.0 - 1.0, 1.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_position_homogenous = camera.inv_projection * in.clip_position;
    let view_ray_direction = view_position_homogenous.xyz / view_position_homogenous.w;
    var ray_direction = normalize((camera.inv_view * vec4(view_ray_direction, 0.0)).xyz);

    let sample = textureSample(env_map, env_sampler, ray_direction);

    return sample;
}
