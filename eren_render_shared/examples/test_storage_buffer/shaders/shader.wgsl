// storage buffer 정의 (binding 0, group 0)
struct StorageBufferObject {
    model : mat4x4<f32>,
    view  : mat4x4<f32>,
    proj  : mat4x4<f32>,
};

@group(0) @binding(0)
var<storage, read> ssbo : StorageBufferObject;

// 정점 입력 구조체
struct VertexInput {
    @location(0) inPosition : vec2<f32>,
    @location(1) inColor    : vec3<f32>,
};

// 정점 출력 구조체
struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) fragColor : vec3<f32>,
};

@vertex
fn vs_main(input : VertexInput) -> VertexOutput {
    var output : VertexOutput;

    let pos = vec4<f32>(input.inPosition, 0.0, 1.0);
    output.position = ssbo.proj * ssbo.view * ssbo.model * pos;

    output.fragColor = input.inColor;
    return output;
}

@fragment
fn fs_main(@location(0) fragColor : vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(fragColor, 1.0);
}
