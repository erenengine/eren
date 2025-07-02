// uniform buffer 정의 (binding 0, group 0)
struct Uniforms {
    model : mat4x4<f32>,
    view  : mat4x4<f32>,
    proj  : mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> ubo : Uniforms;

// 정점 입력 구조체 (vertex buffer layout에서 제공)
struct VertexInput {
    @location(0) inPosition : vec3<f32>,
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
    
    // GLSL과 동일한 방식의 모델-뷰-프로젝션 변환
    let pos = vec4<f32>(input.inPosition, 1.0);
    output.position = ubo.proj * ubo.view * ubo.model * pos;
    
    output.fragColor = input.inColor;
    return output;
}

@fragment
fn fs_main(@location(0) fragColor : vec3<f32>) -> @location(0) vec4<f32> {
    return vec4<f32>(fragColor, 1.0);
}
