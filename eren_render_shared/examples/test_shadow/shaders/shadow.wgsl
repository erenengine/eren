struct ShadowUBO {
    lightViewProj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> ubo: ShadowUBO;

struct VertexInput {
    @location(0) inPosition: vec3<f32>,
    @location(1) inNormal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = ubo.lightViewProj * vec4<f32>(input.inPosition, 1.0);
    return output;
}
