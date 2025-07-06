#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inTexCoord;

layout(set = 0, binding = 0) uniform ShadowUBO {
    mat4 lightViewProj; // 빛의 시점에서의 투영행렬
} ubo;

void main() {
    gl_Position = ubo.lightViewProj * vec4(inPosition, 1.0);
}
