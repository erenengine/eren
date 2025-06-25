struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) fragColor : vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertexIndex : u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, -0.5),
        vec2<f32>(0.5, 0.5),
        vec2<f32>(-0.5, 0.5)
    );

    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );

    var output : VertexOutput;
    output.position = vec4<f32>(
        positions[vertexIndex].x,
        -positions[vertexIndex].y, // Y축 반전
        0.0,
        1.0
    );
    output.fragColor = colors[vertexIndex];
    return output;
}

@fragment
fn fs_main(@location(0) fragColor : vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(fragColor, 1.0);
}
